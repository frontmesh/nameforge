use std::{fs::File, io::Read, path::Path, time::Duration};
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

fn apply_case_conversion(input: &str, case_style: &str) -> String {
    match case_style.to_lowercase().as_str() {
        "snakecase" | "snake_case" => to_snake_case(input),
        "camelcase" | "camel_case" => to_camel_case(input),
        "pascalcase" | "pascal_case" => to_pascal_case(input),
        "kebabcase" | "kebab_case" => to_kebab_case(input),
        _ => input.to_string(), // Return as-is if unknown case style
    }
}

fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    
    for (i, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 && !prev_was_upper {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_upper = true;
        } else if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_was_upper = false;
        } else if ch == ' ' || ch == '-' {
            result.push('_');
            prev_was_upper = false;
        }
        // Skip other special characters
    }
    
    result
}

fn to_camel_case(input: &str) -> String {
    let words: Vec<&str> = input.split(|c: char| !c.is_ascii_alphanumeric()).filter(|s| !s.is_empty()).collect();
    if words.is_empty() {
        return String::new();
    }
    
    let mut result = words[0].to_lowercase();
    for word in &words[1..] {
        if !word.is_empty() {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                result.extend(chars.map(|c| c.to_ascii_lowercase()));
            }
        }
    }
    result
}

fn to_pascal_case(input: &str) -> String {
    let words: Vec<&str> = input.split(|c: char| !c.is_ascii_alphanumeric()).filter(|s| !s.is_empty()).collect();
    let mut result = String::new();
    
    for word in words {
        if !word.is_empty() {
            let mut chars = word.chars();
            if let Some(first) = chars.next() {
                result.push(first.to_ascii_uppercase());
                result.extend(chars.map(|c| c.to_ascii_lowercase()));
            }
        }
    }
    result
}

fn to_kebab_case(input: &str) -> String {
    to_snake_case(input).replace('_', "-")
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
    
    // Make request to Ollama with timeout and retry
    let client = Client::builder()
        .timeout(Duration::from_secs(30)) // 30 second timeout for model loading
        .build()
        .unwrap();
    
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        images: vec![base64_image],
        stream: false,
    };
    
    println!("{}  {}{}{}", "🤖".bright_magenta(), "Analyzing image content with AI model: ".bright_magenta(), model.bright_white().bold(), "...".bright_magenta());
    
    // Try up to 2 times with a brief pause between attempts
    let mut last_error = None;
    for attempt in 1..=2 {
        let response = client
            .post("http://localhost:11434/api/generate")
            .json(&request)
            .send();
            
        match response {
            Ok(resp) => {
                if attempt > 1 {
                    println!("{}  {}{}", "✅".bright_green(), "Retry successful on attempt ".bright_green(), attempt.to_string().bright_white());
                }
                
                if !resp.status().is_success() {
                    eprintln!("{} {}{}{}  {}{}", "❌".bright_red(), "Ollama API error status: ".bright_red(), resp.status().to_string().bright_white().bold(), " - ".bright_red(), "Details: ".bright_red(), resp.text().unwrap_or_default().bright_white());
                    return None;
                }
                
                let ollama_response: OllamaResponse = match resp.json() {
                    Ok(resp) => resp,
                    Err(e) => {
                        eprintln!("{} {}{}", "❌".bright_red(), "Failed to parse Ollama response: ".bright_red(), e.to_string().bright_white());
                        return None;
                    }
                };
                
                // Clean the response (remove any extra whitespace, newlines, etc.)
                let mut filename = ollama_response.response.trim().to_string();
                
                if filename.is_empty() {
                    eprintln!("{} {}", "❌".bright_red(), "Ollama returned empty response".bright_red());
                    return None;
                }
                
                // Apply case conversion to ensure consistency
                filename = apply_case_conversion(&filename, case);
                
                println!("{}  {}{}{}{}", "✨".bright_yellow(), "AI generated filename: ".bright_yellow(), "'".bright_white(), filename.bright_green().bold(), "'".bright_white());
                
                // Validate the response length
                if filename.len() > max_chars as usize {
                    return Some(filename.chars().take(max_chars as usize).collect());
                } else {
                    return Some(filename);
                }
            },
            Err(e) => {
                last_error = Some(e);
                if attempt == 1 {
                    println!("{} {}  {}", "⚠️".bright_yellow(), "First attempt failed, retrying...".bright_yellow(), "(model might be loading)".bright_black());
                    std::thread::sleep(Duration::from_millis(2000)); // Wait 2 seconds before retry
                }
            }
        }
    }
    
    // If we get here, both attempts failed
    if let Some(e) = last_error {
        eprintln!("{} {}{}", "❌".bright_red(), "Failed to send request to Ollama after 2 attempts: ".bright_red(), e.to_string().bright_white());
    }
    None
}