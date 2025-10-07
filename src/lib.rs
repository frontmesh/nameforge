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

/// Check if buffer contains valid image file signature
fn is_valid_image_signature(buffer: &[u8; 4]) -> bool {
    matches!(buffer,
        [0xFF, 0xD8, _, _] |      // JPEG
        [0x89, 0x50, 0x4E, 0x47] | // PNG
        [0x47, 0x49, 0x46, 0x38] | // GIF87a
        [0x47, 0x49, 0x46, 0x39] | // GIF89a
        [0x42, 0x4D, _, _] |      // BMP
        [0x52, 0x49, 0x46, 0x46]   // WEBP (starts with RIFF)
    )
}

/// Quick validation to check if file is a valid image by reading the first few bytes
fn is_valid_image_file(image_path: &Path) -> bool {
    use std::fs::File;
    use std::io::Read;
    
    File::open(image_path)
        .ok()
        .and_then(|mut file| {
            let mut buffer = [0u8; 4];
            file.read_exact(&mut buffer).ok().map(|_| buffer)
        })
        .map(|buffer| is_valid_image_signature(&buffer))
        .unwrap_or(false)
}

pub fn process_folder(
    input_path: &Path, 
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
    /// Helper function to check if a file is a valid image file
    fn is_valid_image(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(is_supported_image_extension)
            .unwrap_or(false) && is_valid_image_file(path)
    }
    
    /// Helper function to filter out macOS resource fork files
    fn is_not_resource_fork(path: &Path) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| !name.starts_with("._"))
            .unwrap_or(true)
    }
    
    // Collect all valid image files using functional approach
    let image_files: Vec<_> = if input_path.is_file() {
        // Process a single file
        if is_valid_image(input_path) {
            vec![input_path.to_path_buf()]
        } else {
            eprintln!("{} {}{}", "❌".bright_red(), "Not a valid image file: ".bright_red(), input_path.display().to_string().bright_white());
            return;
        }
    } else if input_path.is_dir() {
        // Process directory using functional approach
        match fs::read_dir(input_path) {
            Ok(entries) => {
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| is_not_resource_fork(path))
                    .filter(|path| is_valid_image(path))
                    .collect()
            },
            Err(e) => {
                eprintln!("Could not open folder {:?}: {}", input_path, e);
                return;
            }
        }
    } else {
        eprintln!("{} {}{}", "❌".bright_red(), "Input path does not exist or is not accessible: ".bright_red(), input_path.display().to_string().bright_white());
        return;
    };
    
    let total_files = image_files.len();
    println!("{}  {}{}{}", "📊".bright_blue(), "Found ".bright_blue(), total_files.to_string().bright_white().bold(), " valid image files to process".bright_blue());
    
    let mut gps_cache = GPSCache::load();
    let mut cache_updated = false;
    let mut processed_count = 0;

    // Second pass: process the collected files
    for path in image_files {
        if max_images.map(|max| processed_count >= max).unwrap_or(false) {
            println!("{}  {}{}{}", "🎯".bright_cyan(), "Reached maximum image limit of ".bright_cyan(), max_images.unwrap().to_string().bright_white().bold(), ". Stopping processing.".bright_cyan());
            break;
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
            cache_updated |= updated;

            let base_folder = get_base_folder(input_path);
            let new_path = get_target_path(&path, base_folder, &new_name, organize_by_date);

            if dry_run {
                print_dry_run_info(&path, &new_path);
            } else {
                if let Err(e) = execute_rename(&path, &new_path) {
                    eprintln!("{} {}{} {} {}{}  {}{}", "❌".bright_red(), "Failed to rename ".bright_red(), path.display().to_string().bright_white(), "→".bright_red(), new_path.display().to_string().bright_white(), ": ".bright_red(), "Error: ".bright_red(), e.to_string().bright_white());
                }
            }
        }

        processed_count += 1;
    }
    
    // Save cache if it was updated
    if cache_updated {
        gps_cache.save();
    }
}

/// Helper function to resolve GPS location with fallback  
fn resolve_gps_location(exif_opt: &Option<::exif::Exif>, cache: &mut GPSCache) -> (String, bool) {
    exif_opt
        .as_ref()
        .and_then(extract_gps_coordinates)
        .map(|(lat, lon)| {
            let (place_result, updated) = gps_to_place(lat, lon, cache);
            (place_result.unwrap_or_else(|| "UnknownPlace".to_string()), updated)
        })
        .unwrap_or_else(|| ("NoGPS".to_string(), false))
}

/// Helper function to generate AI content with date fallback
fn generate_ai_content(
    path: &Path,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
    exif_opt: &Option<::exif::Exif>,
    use_file_date: bool,
    prefer_modified: bool,
) -> String {
    get_ai_content_name(path, ai_model, ai_max_chars, ai_case, ai_language)
        .unwrap_or_else(|| {
            let fallback_date = get_date_string(path, exif_opt, false, use_file_date, prefer_modified)
                .unwrap_or_else(|| {
                    use chrono::Local;
                    Local::now().format("%Y-%m-%d_%H-%M-%S").to_string()
                });
            eprintln!(
                "{} {}{}  {}{}",
                "⚠️".bright_yellow(),
                "Failed to get AI content analysis for ".bright_yellow(),
                path.display().to_string().bright_white(),
                "using date fallback: ".bright_yellow(),
                fallback_date.bright_white()
            );
            fallback_date
        })
}

/// Helper function to create base filename from date and content
fn create_base_filename(date_fmt: Option<String>, content_part: String) -> String {
    date_fmt
        .map(|date| format!("{}_{}", date, content_part))
        .unwrap_or(content_part)
}

/// Helper function to get base folder for file operations
fn get_base_folder(input_path: &Path) -> &Path {
    if input_path.is_dir() {
        input_path
    } else {
        input_path.parent().unwrap_or(Path::new("."))
    }
}

/// Helper function to determine target path for renamed file
fn get_target_path(original_path: &Path, base_folder: &Path, new_name: &str, organize_by_date: bool) -> std::path::PathBuf {
    if organize_by_date {
        create_date_folder_path(base_folder, new_name)
    } else {
        original_path.with_file_name(new_name)
    }
}

/// Helper function to print dry run information
fn print_dry_run_info(original_path: &Path, new_path: &Path) {
    println!(
        "{}  {}{} {} {}",
        "💁".bright_yellow(),
        "Dry run: ".bright_yellow().bold(),
        original_path.display().to_string().bright_white(),
        "→".bright_yellow(),
        new_path.display().to_string().bright_green()
    );
}

/// Helper function to execute file rename with error handling
fn execute_rename(original_path: &Path, new_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create the directory if it doesn't exist (for date folders)
    if let Some(parent) = new_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                eprintln!(
                    "{} {}{}{}  {}{}",
                    "❌".bright_red(),
                    "Failed to create directory ".bright_red(),
                    parent.display().to_string().bright_white(),
                    ": ".bright_red(),
                    "Error: ".bright_red(),
                    e.to_string().bright_white()
                );
                e
            })?;
        }
    }

    println!(
        "{}  {}{} {} {}",
        "🔄".bright_green(),
        "Renaming: ".bright_green(),
        original_path.display().to_string().bright_white(),
        "→".bright_green(),
        new_path.display().to_string().bright_green().bold()
    );

    fs::rename(original_path, new_path).map_err(|e| {
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    println!(
        "{} {}{} {} {}",
        "✅".bright_green(),
        "Successfully renamed: ".bright_green(),
        original_path.display().to_string().bright_white(),
        "→".bright_green(),
        new_path.display().to_string().bright_green().bold()
    );

    Ok(())
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
    let date_fmt = (!no_date)
        .then(|| get_date_string(path, &exif_opt, date_only, use_file_date, prefer_modified))
        .flatten();
    let ext = path.extension()?.to_str().unwrap_or("jpg");
    let folder = path.parent()?;

    // Generate content and track GPS cache updates
    let (content_part, gps_cache_updated) = if ai_content {
        let ai_content = generate_ai_content(
            path, ai_model, ai_max_chars, ai_case, ai_language,
            &exif_opt, use_file_date, prefer_modified
        );
        (ai_content, false) // AI content doesn't use GPS cache
    } else {
        let (place, updated) = resolve_gps_location(&exif_opt, cache);
        (place, updated)
    };
    
    let base_name = create_base_filename(date_fmt, content_part);
    unique_filename(folder, &base_name, ext).map(|name| (name, gps_cache_updated))
}
