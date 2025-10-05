use std::{path::Path, time::Duration};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use base64::{Engine as _, engine::general_purpose};
use colored::*;
use image::ImageFormat;
use std::io::Cursor;

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
        "lowercase" => {
            // For lowercase, still convert spaces/hyphens to underscores and make lowercase
            input.chars().filter_map(|c| {
                if c == ' ' || c == '-' {
                    Some('_')
                } else if c.is_ascii_alphanumeric() {
                    Some(c.to_ascii_lowercase())
                } else {
                    // Skip other special characters
                    None
                }
            }).collect::<String>()
        },
        "uppercase" => {
            // For uppercase, convert spaces/hyphens to underscores and make uppercase
            input.chars().filter_map(|c| {
                if c == ' ' || c == '-' {
                    Some('_')
                } else if c.is_ascii_alphanumeric() {
                    Some(c.to_ascii_uppercase())
                } else {
                    // Skip other special characters
                    None
                }
            }).collect::<String>()
        },
        _ => to_snake_case(input), // Default to snake_case for unknown styles
    }
}

fn to_snake_case(input: &str) -> String {
    let mut result = String::new();
    let mut prev_was_upper = false;
    let mut prev_was_separator = false;
    
    for (i, ch) in input.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            // Add underscore before uppercase letter if previous wasn't uppercase and we're not at start
            if i > 0 && !prev_was_upper && !prev_was_separator {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
            prev_was_upper = true;
            prev_was_separator = false;
        } else if ch.is_ascii_alphanumeric() {
            result.push(ch);
            prev_was_upper = false;
            prev_was_separator = false;
        } else if ch == ' ' || ch == '-' || ch == '_' {
            // Only add underscore if the last character wasn't already a separator
            if !prev_was_separator && !result.is_empty() {
                result.push('_');
            }
            prev_was_upper = false;
            prev_was_separator = true;
        }
        // Skip other special characters
    }
    
    // Remove trailing underscore if present
    if result.ends_with('_') {
        result.pop();
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

/// Resize image to reduce memory usage while maintaining aspect ratio
fn resize_image_for_ai(image_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Open and decode the image
    let img = image::open(image_path)?;
    
    // Calculate new dimensions (max 1024px on longest side)
    let max_size = 1024;
    let (width, height) = (img.width(), img.height());
    let (new_width, new_height) = if width > height {
        if width > max_size {
            let ratio = max_size as f32 / width as f32;
            (max_size, (height as f32 * ratio) as u32)
        } else {
            (width, height)
        }
    } else {
        if height > max_size {
            let ratio = max_size as f32 / height as f32;
            ((width as f32 * ratio) as u32, max_size)
        } else {
            (width, height)
        }
    };
    
    // Resize the image
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    
    // Encode to JPEG with reduced quality
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    resized.write_to(&mut cursor, ImageFormat::Jpeg)?;
    
    Ok(buffer)
}

pub fn get_ai_content_name(
    image_path: &Path,
    model: &str,
    max_chars: u32,
    case: &str,
    language: &str,
) -> Option<String> {
    // Resize and encode the image to reduce memory usage
    println!("{}  {}{}", "🖼️".bright_blue(), "Resizing image for AI processing...".bright_blue(), "");
    let buffer = match resize_image_for_ai(image_path) {
        Ok(buf) => buf,
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("invalid JPEG format") || error_msg.contains("Format error") {
                eprintln!("{} {}{}{}", "⚠️".bright_yellow(), "Skipping invalid/corrupted image file: ".bright_yellow(), image_path.display().to_string().bright_white(), " (not a valid image format)".bright_yellow());
            } else {
                eprintln!("{} {}{}", "❌".bright_red(), "Failed to resize image: ".bright_red(), error_msg.bright_white());
            }
            return None;
        }
    };
    let base64_image = general_purpose::STANDARD.encode(&buffer);
    
    // Build the prompt according to specification
    let prompt = format!(
        "Look at this image and generate a descriptive filename.\n\nRules:\n- Use {} case format{}\n- Maximum {} characters\n- {} language only\n- No file extension\n- No special characters except underscores\n- Describe the main subject/action\n- Be concise and specific\n\nRespond with ONLY the filename, nothing else.",
        case, 
        if case.to_lowercase().contains("snake") { " (separate_words_with_underscores)" } else { "" },
        max_chars, 
        language
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("cat on carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("CatOnCarpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("catOnCarpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("cat-on-carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("cat_on_carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("CAT ON CARPET"), "cat_on_carpet");
        assert_eq!(to_snake_case("cat  on   carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("cat--on--carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("cat__on__carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("Cat On Carpet"), "cat_on_carpet");
        assert_eq!(to_snake_case("catoncarpet"), "catoncarpet");
        assert_eq!(to_snake_case("CATONCARPET"), "catoncarpet");
    }

    #[test]
    fn test_apply_case_conversion() {
        assert_eq!(apply_case_conversion("cat on carpet", "snake_case"), "cat_on_carpet");
        assert_eq!(apply_case_conversion("cat on carpet", "snakecase"), "cat_on_carpet");
        assert_eq!(apply_case_conversion("cat on carpet", "lowercase"), "cat_on_carpet");
        assert_eq!(apply_case_conversion("cat on carpet", "uppercase"), "CAT_ON_CARPET");
        assert_eq!(apply_case_conversion("cat on carpet", "camelcase"), "catOnCarpet");
        assert_eq!(apply_case_conversion("cat on carpet", "pascalcase"), "CatOnCarpet");
        assert_eq!(apply_case_conversion("cat on carpet", "kebabcase"), "cat-on-carpet");
        // Unknown case should default to snake_case
        assert_eq!(apply_case_conversion("cat on carpet", "unknown"), "cat_on_carpet");
    }
}
