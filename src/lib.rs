mod ai;
mod cache;
mod exif;
mod gps;
mod utils;

use std::{fs, path::Path};
use cache::GPSCache;
use exif::{extract_gps_coordinates, get_date_string, read_exif_data};
use gps::gps_to_place;
use ai::get_ai_content_name;
use utils::{create_date_folder_path, unique_filename};
use colored::*;

// Supported image file extensions
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "bmp", "webp", "heic", "heif", "raw", "cr2", "nef", "arw"
];

/// Check if the given file extension is a supported image format
fn is_supported_image_extension(extension: &str) -> bool {
    SUPPORTED_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
}

/// Quick validation to check if file is a valid image by reading the first few bytes
fn is_valid_image_file(image_path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;
    
    let mut file = match File::open(image_path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    
    let mut buffer = [0u8; 4];
    if file.read_exact(&mut buffer).is_err() {
        return false;
    }
    
    // Check for common image file signatures
    match &buffer {
        [0xFF, 0xD8, _, _] => true, // JPEG
        [0x89, 0x50, 0x4E, 0x47] => true, // PNG
        [0x47, 0x49, 0x46, 0x38] => true, // GIF87a
        [0x47, 0x49, 0x46, 0x39] => true, // GIF89a
        [0x42, 0x4D, _, _] => true, // BMP
        [0x52, 0x49, 0x46, 0x46] => true, // WEBP (starts with RIFF)
        _ => false,
    }
}

pub fn process_folder(
    folder: &Path, 
    dry_run: bool, 
    organize_by_date: bool,
    ai_content: bool,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
    date_only: bool,
    max_images: Option<usize>,
    use_file_date: bool,
    prefer_modified: bool,
    no_date: bool,
) {
    // Collect all valid image files first to avoid issues with directory modification during iteration
    let mut image_files = Vec::new();
    
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Could not open folder {:?}: {}", folder, e);
            return;
        }
    };
    
    // First pass: collect all valid image files
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                // Skip macOS resource fork files (._filename)
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("._") {
                        continue; // Skip resource fork files
                    }
                }
                
                if is_supported_image_extension(ext_str) {
                    // Validate that it's actually a valid image file
                    if is_valid_image_file(&path) {
                        image_files.push(path);
                    }
                }
            }
        }
    }
    
    let total_files = image_files.len();
    println!("{}  {}{}{}", "📊".bright_blue(), "Found ".bright_blue(), total_files.to_string().bright_white().bold(), " valid image files to process".bright_blue());
    
    let mut gps_cache = GPSCache::load();
    let mut cache_updated = false;
    let mut processed_count = 0;

    // Second pass: process the collected files
    for path in image_files {
        // Check if we've reached the max_images limit
        if let Some(max) = max_images {
            if processed_count >= max {
                println!("{}  {}{}{}", "🎯".bright_cyan(), "Reached maximum image limit of ".bright_cyan(), max.to_string().bright_white().bold(), ". Stopping processing.".bright_cyan());
                break;
            }
        }
        
        println!("{}  {}{}", "📷".bright_blue(), "Processing image file: ".bright_blue(), path.display().to_string().bright_white().bold());
        
        if let Some((new_name, updated)) = build_new_name(
            &path, 
            &mut gps_cache, 
            ai_content, 
            ai_model, 
            ai_max_chars, 
            ai_case, 
            ai_language,
            date_only,
            use_file_date,
            prefer_modified,
            no_date,
        ) {
            if updated {
                cache_updated = true;
            }
            
            let new_path = if organize_by_date {
                create_date_folder_path(folder, &new_name)
            } else {
                path.with_file_name(new_name)
            };
            
            if dry_run {
                println!("{}  {}{} {} {}", "💁".bright_yellow(), "Dry run: ".bright_yellow().bold(), path.display().to_string().bright_white(), "→".bright_yellow(), new_path.display().to_string().bright_green());
            } else {
                // Create the directory if it doesn't exist (for date folders)
                if let Some(parent) = new_path.parent() {
                    if !parent.exists() {
                        if let Err(e) = fs::create_dir_all(parent) {
                            eprintln!("{} {}{}{}  {}{}", "❌".bright_red(), "Failed to create directory ".bright_red(), parent.display().to_string().bright_white(), ": ".bright_red(), "Error: ".bright_red(), e.to_string().bright_white());
                            continue;
                        }
                    }
                }
                
                println!("{}  {}{} {} {}", "🔄".bright_green(), "Renaming: ".bright_green(), path.display().to_string().bright_white(), "→".bright_green(), new_path.display().to_string().bright_green().bold());
                if let Err(e) = fs::rename(&path, &new_path) {
                    eprintln!("{} {}{} {} {}{}  {}{}", "❌".bright_red(), "Failed to rename ".bright_red(), path.display().to_string().bright_white(), "→".bright_red(), new_path.display().to_string().bright_white(), ": ".bright_red(), "Error: ".bright_red(), e.to_string().bright_white());
                } else {
                    println!("{} {}{} {} {}", "✅".bright_green(), "Successfully renamed: ".bright_green(), path.display().to_string().bright_white(), "→".bright_green(), new_path.display().to_string().bright_green().bold());
                }
            }
        }
        
        // Increment processed count regardless of dry run or success
        processed_count += 1;
    }
    
    // Save cache if it was updated
    if cache_updated {
        gps_cache.save();
    }
}

fn build_new_name(
    path: &Path, 
    cache: &mut GPSCache,
    ai_content: bool,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
    date_only: bool,
    use_file_date: bool,
    prefer_modified: bool,
    no_date: bool,
) -> Option<(String, bool)> {
    let exif_opt = read_exif_data(path);
    let date_fmt = if no_date {
        None
    } else {
        get_date_string(path, &exif_opt, date_only, use_file_date, prefer_modified)
    };
    let ext = path.extension()?.to_str().unwrap_or("jpg");
    let folder = path.parent()?;

    // Only resolve GPS coordinates if AI content is NOT being used
    let (place, gps_cache_updated) = if ai_content {
        // Skip GPS resolution when using AI content
        ("NoGPS".to_string(), false)
    } else if let Some(exif) = &exif_opt {
        if let Some((lat, lon)) = extract_gps_coordinates(exif) {
            let (place_result, updated) = gps_to_place(lat, lon, cache);
            (place_result.unwrap_or_else(|| "UnknownPlace".to_string()), updated)
        } else {
            ("NoGPS".to_string(), false)
        }
    } else {
        ("NoGPS".to_string(), false)
    };

    // Generate content description using AI if enabled
    let content_part = if ai_content {
        match get_ai_content_name(path, ai_model, ai_max_chars, ai_case, ai_language) {
            Some(ai_name) => ai_name,
            None => {
                eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "Failed to get AI content analysis for ".bright_yellow(), path.display().to_string().bright_white(), "using 'Content' fallback".bright_yellow());
                "Content".to_string()
            }
        }
    } else {
        place.clone()
    };
    
    let base_name = if let Some(date) = date_fmt {
        format!("{}_{}", date, content_part)
    } else {
        content_part
    };
    
    unique_filename(folder, &base_name, ext).map(|name| (name, gps_cache_updated))
}
