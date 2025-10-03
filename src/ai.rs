use std::{fs::File, io::Read, path::Path};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use colored::*;

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

pub fn get_ai_content_name(
    image_path: &Path,
    model: &str,
    max_chars: u32,
    case: &str,
    language: &str,
) -> Option<String> {
    // Read and encode the image
    let mut file = File::open(image_path).ok()?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).ok()?;
    let base64_image = general_purpose::STANDARD.encode(&buffer);
    
    // Build the prompt according to specification
    let prompt = format!(
        "Generate filename:\n\nUse {}\nMax {} characters\n{} only\nNo file extension\nNo special chars\nOnly key elements\nOne word if possible\nNoun-verb format\n\nRespond ONLY with filename.",
        case, max_chars, language
    );
    
    // Make request to Ollama
    let client = Client::new();
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        images: vec![base64_image],
        stream: false,
    };
    
    println!("{}  {}{}{}", "🤖".bright_magenta(), "Analyzing image content with AI model: ".bright_magenta(), model.bright_white().bold(), "...".bright_magenta());
    
    let response = match client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("{} {}{}", "❌".bright_red(), "Failed to send request to Ollama: ".bright_red(), e.to_string().bright_white());
            return None;
        }
    };
    
    if !response.status().is_success() {
        eprintln!("{} {}{}{}  {}{}", "❌".bright_red(), "Ollama API error status: ".bright_red(), response.status().to_string().bright_white().bold(), " - ".bright_red(), "Details: ".bright_red(), response.text().unwrap_or_default().bright_white());
        return None;
    }
    
    let ollama_response: OllamaResponse = match response.json() {
        Ok(resp) => resp,
        Err(e) => {
            eprintln!("{} {}{}", "❌".bright_red(), "Failed to parse Ollama response: ".bright_red(), e.to_string().bright_white());
            return None;
        }
    };
    
    // Clean the response (remove any extra whitespace, newlines, etc.)
    let filename = ollama_response.response.trim().to_string();
    
    if filename.is_empty() {
        eprintln!("{} {}", "❌".bright_red(), "Ollama returned empty response".bright_red());
        return None;
    }
    
    println!("{}  {}{}{}{}", "✨".bright_yellow(), "AI generated filename: ".bright_yellow(), "'".bright_white(), filename.bright_green().bold(), "'".bright_white());
    
    // Validate the response length
    if filename.len() > max_chars as usize {
        Some(filename.chars().take(max_chars as usize).collect())
    } else {
        Some(filename)
    }
}