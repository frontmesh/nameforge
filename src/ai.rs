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
        "lowercase" => normalize_to_underscore_case(input, |c| c.to_ascii_lowercase()),
        "uppercase" => normalize_to_underscore_case(input, |c| c.to_ascii_uppercase()),
        _ => to_snake_case(input), // Default to snake_case for unknown styles
    }
}

/// Helper function to normalize input to underscore-separated case
fn normalize_to_underscore_case<F>(input: &str, transform: F) -> String
where
    F: Fn(char) -> char,
{
    input
        .chars()
        .filter_map(|c| match c {
            ' ' | '-' => Some('_'),
            c if c.is_ascii_alphanumeric() => Some(transform(c)),
            _ => None, // Skip other special characters
        })
        .collect()
}

/// Helper to split input into alphanumeric words
fn split_into_words(input: &str) -> Vec<&str> {
    input
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Helper to capitalize first letter and lowercase the rest
fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_ascii_uppercase().to_string() + &chars.map(|c| c.to_ascii_lowercase()).collect::<String>(),
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
    let words = split_into_words(input);
    words
        .into_iter()
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_lowercase()
            } else {
                capitalize_word(word)
            }
        })
        .collect()
}

fn to_pascal_case(input: &str) -> String {
    split_into_words(input)
        .into_iter()
        .map(capitalize_word)
        .collect()
}

fn to_kebab_case(input: &str) -> String {
    to_snake_case(input).replace('_', "-")
}

/// Calculate new image dimensions maintaining aspect ratio
fn calculate_resize_dimensions(width: u32, height: u32, max_size: u32) -> (u32, u32) {
    if width.max(height) <= max_size {
        (width, height)
    } else {
        let ratio = max_size as f32 / width.max(height) as f32;
        (
            (width as f32 * ratio) as u32,
            (height as f32 * ratio) as u32
        )
    }
}

/// Encode image to JPEG buffer
fn encode_image_to_jpeg(img: image::DynamicImage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    img.write_to(&mut cursor, ImageFormat::Jpeg)?;
    Ok(buffer)
}

/// Resize image to reduce memory usage while maintaining aspect ratio
fn resize_image_for_ai(image_path: &Path) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    const MAX_SIZE: u32 = 1024;
    
    let img = image::open(image_path)?;
    let (width, height) = (img.width(), img.height());
    let (new_width, new_height) = calculate_resize_dimensions(width, height, MAX_SIZE);
    let resized = img.resize(new_width, new_height, image::imageops::FilterType::Lanczos3);
    encode_image_to_jpeg(resized)
}

/// Helper function to prepare image for AI processing
fn prepare_image_for_ai(image_path: &Path) -> Option<String> {
    println!("{}  {}{}", "🖼️".bright_blue(), "Resizing image for AI processing...".bright_blue(), "");
    
    resize_image_for_ai(image_path)
        .map(|buffer| general_purpose::STANDARD.encode(&buffer))
        .map_err(|e| {
            let error_msg = e.to_string();
            if error_msg.contains("invalid JPEG format") || error_msg.contains("Format error") {
                eprintln!(
                    "{} {}{}{}",
                    "⚠️".bright_yellow(),
                    "Skipping invalid/corrupted image file: ".bright_yellow(),
                    image_path.display().to_string().bright_white(),
                    " (not a valid image format)".bright_yellow()
                );
            } else {
                eprintln!(
                    "{} {}{}",
                    "❌".bright_red(),
                    "Failed to resize image: ".bright_red(),
                    error_msg.bright_white()
                );
            }
        })
        .ok()
}

/// Helper function to build AI prompt
fn build_ai_prompt(case: &str, max_chars: u32, language: &str) -> String {
    format!(
        "Look at this image and generate a descriptive filename.\n\nRules:\n- Use {} case format{}\n- Maximum {} characters\n- {} language only\n- No file extension\n- No special characters except underscores\n- Describe the main subject/action\n- Be concise and specific\n\nRespond with ONLY the filename, nothing else.",
        case,
        if case.to_lowercase().contains("snake") { " (separate_words_with_underscores)" } else { "" },
        max_chars,
        language
    )
}

/// Helper function to create HTTP client
fn create_ai_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap()
}

/// Helper function to process successful AI response
fn process_ai_response(response: reqwest::blocking::Response, case: &str, max_chars: u32) -> Option<String> {
    if !response.status().is_success() {
        eprintln!(
            "{} {}{}{}  {}{}",
            "❌".bright_red(),
            "Ollama API error status: ".bright_red(),
            response.status().to_string().bright_white().bold(),
            " - ".bright_red(),
            "Details: ".bright_red(),
            response.text().unwrap_or_default().bright_white()
        );
        return None;
    }
    
    let ollama_response: OllamaResponse = response.json().map_err(|e| {
        eprintln!(
            "{} {}{}",
            "❌".bright_red(),
            "Failed to parse Ollama response: ".bright_red(),
            e.to_string().bright_white()
        );
    }).ok()?;
    
    let filename = ollama_response.response.trim();
    if filename.is_empty() {
        eprintln!("{} {}", "❌".bright_red(), "Ollama returned empty response".bright_red());
        return None;
    }
    
    let filename = apply_case_conversion(filename, case);
    println!(
        "{}  {}{}{}{}",
        "✨".bright_yellow(),
        "AI generated filename: ".bright_yellow(),
        "'".bright_white(),
        filename.bright_green().bold(),
        "'".bright_white()
    );
    
    Some(if filename.len() > max_chars as usize {
        filename.chars().take(max_chars as usize).collect()
    } else {
        filename
    })
}

/// Helper function to attempt AI request with retry logic
fn attempt_ai_request(client: &Client, request: &OllamaRequest, case: &str, max_chars: u32) -> Option<String> {
    println!(
        "{}  {}{}{}",
        "🤖".bright_magenta(),
        "Analyzing image content with AI model: ".bright_magenta(),
        request.model.bright_white().bold(),
        "...".bright_magenta()
    );
    
    for attempt in 1..=2 {
        match client.post("http://localhost:11434/api/generate").json(request).send() {
            Ok(response) => {
                if attempt > 1 {
                    println!(
                        "{}  {}{}",
                        "✅".bright_green(),
                        "Retry successful on attempt ".bright_green(),
                        attempt.to_string().bright_white()
                    );
                }
                return process_ai_response(response, case, max_chars);
            }
            Err(e) => {
                if attempt == 1 {
                    println!(
                        "{} {}  {}",
                        "⚠️".bright_yellow(),
                        "First attempt failed, retrying...".bright_yellow(),
                        "(model might be loading)".bright_black()
                    );
                    std::thread::sleep(Duration::from_millis(2000));
                } else {
                    eprintln!(
                        "{} {}{}",
                        "❌".bright_red(),
                        "Failed to send request to Ollama after 2 attempts: ".bright_red(),
                        e.to_string().bright_white()
                    );
                    return None;
                }
            }
        }
    }
    None
}

pub fn get_ai_content_name(
    image_path: &Path,
    model: &str,
    max_chars: u32,
    case: &str,
    language: &str,
) -> Option<String> {
    let base64_image = prepare_image_for_ai(image_path)?;
    let client = create_ai_client();
    let prompt = build_ai_prompt(case, max_chars, language);
    
    let request = OllamaRequest {
        model: model.to_string(),
        prompt,
        images: vec![base64_image],
        stream: false,
    };
    
    attempt_ai_request(&client, &request, case, max_chars)
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
