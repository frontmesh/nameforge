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

pub fn process_folder(
    folder: &Path, 
    dry_run: bool, 
    organize_by_date: bool,
    ai_content: bool,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
) {
    let entries = match fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Could not open folder {:?}: {}", folder, e);
            return;
        }
    };

    let mut gps_cache = GPSCache::load();
    let mut cache_updated = false;

    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if let Some(ext) = path.extension() {
            if ext.to_ascii_lowercase() == "jpg" {
                println!("{}  {}{}", "📷".bright_blue(), "Processing JPEG file: ".bright_blue(), path.display().to_string().bright_white().bold());
                
                if let Some((new_name, updated)) = build_new_name(
                    &path, 
                    &mut gps_cache, 
                    ai_content, 
                    ai_model, 
                    ai_max_chars, 
                    ai_case, 
                    ai_language
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
            }
        }
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
) -> Option<(String, bool)> {
    let exif_opt = read_exif_data(path);
    let date_fmt = get_date_string(path, &exif_opt)?;
    let ext = path.extension()?.to_str().unwrap_or("jpg");
    let folder = path.parent()?;

    let (place, gps_cache_updated) = if let Some(exif) = &exif_opt {
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
    
    let base_name = format!("{}_{}", date_fmt, content_part);
    
    unique_filename(folder, &base_name, ext).map(|name| (name, gps_cache_updated))
}
