use std::{fs, io::BufReader, path::Path};
use chrono::{NaiveDateTime, DateTime, Local};
use exif::{Reader, Tag, In, Field, Value};
use colored::*;

pub fn parse_gps_rational(field: Option<&Field>) -> Option<f64> {
    field
        .and_then(|f| match &f.value {
            Value::Rational(vec) if vec.len() >= 3 => {
                let deg = vec[0].to_f64();
                let min = vec[1].to_f64();
                let sec = vec[2].to_f64();
                Some(deg + min / 60.0 + sec / 3600.0)
            }
            _ => None
        })
}

/// Helper function to get file system time based on preference
fn get_file_time(metadata: &fs::Metadata, prefer_modified: bool) -> Option<std::time::SystemTime> {
    let (primary, fallback) = if prefer_modified {
        (metadata.modified(), metadata.created())
    } else {
        (metadata.created(), metadata.modified())
    };
    
    primary.ok().or_else(|| fallback.ok())
}

/// Helper function to format system time as string
fn format_system_time(time: std::time::SystemTime, date_only: bool) -> String {
    let dt: DateTime<Local> = time.into();
    let format_str = if date_only { "%Y-%m-%d" } else { "%Y-%m-%d_%H-%M-%S" };
    dt.format(format_str).to_string()
}

/// Helper function to try parsing EXIF date
fn try_parse_exif_date(exif: &exif::Exif, date_only: bool) -> Option<String> {
    exif.get_field(Tag::DateTimeOriginal, In::PRIMARY)
        .map(|field| field.display_value().with_unit(exif).to_string())
        .and_then(|date_str| {
            NaiveDateTime::parse_from_str(&date_str, "%Y:%m:%d %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M:%S"))
                .ok()
        })
        .map(|date| {
            let format_str = if date_only { "%Y-%m-%d" } else { "%Y-%m-%d_%H-%M-%S" };
            date.format(format_str).to_string()
        })
}

pub fn get_date_string(path: &Path, exif_opt: &Option<exif::Exif>, date_only: bool, use_file_date: bool, prefer_modified: bool) -> Option<String> {
    // Get file metadata once
    let metadata = fs::metadata(path).ok()?;
    
    // If use_file_date is true, prioritize file system date
    if use_file_date {
        return get_file_time(&metadata, prefer_modified)
            .map(|time| format_system_time(time, date_only));
    }
    
    // Try EXIF date first, with appropriate fallback messages
    let exif_result = exif_opt
        .as_ref()
        .and_then(|exif| try_parse_exif_date(exif, date_only));
        
    match (exif_opt, exif_result) {
        (None, _) => {
            eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "No EXIF data for ".bright_yellow(), path.display().to_string().bright_white(), "falling back to file modified time".bright_yellow());
        },
        (Some(_), None) => {
            eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "No EXIF DateTimeOriginal for ".bright_yellow(), path.display().to_string().bright_white(), "falling back to file modified time".bright_yellow());
        },
        (Some(_), Some(date)) => return Some(date),
    }
    
    // Fallback to file system date
    get_file_time(&metadata, prefer_modified)
        .map(|time| format_system_time(time, date_only))
}

pub fn extract_gps_coordinates(exif: &exif::Exif) -> Option<(f64, f64)> {
    let lat_val = exif.get_field(Tag::GPSLatitude, In::PRIMARY);
    let lon_val = exif.get_field(Tag::GPSLongitude, In::PRIMARY);

    let lat_ref = exif
        .get_field(Tag::GPSLatitudeRef, In::PRIMARY)
        .and_then(|f| match &f.value {
            Value::Ascii(vec) if !vec.is_empty() && !vec[0].is_empty() => Some(vec[0][0] as char),
            _ => None,
        })
        .unwrap_or('N');

    let lon_ref = exif
        .get_field(Tag::GPSLongitudeRef, In::PRIMARY)
        .and_then(|f| match &f.value {
            Value::Ascii(vec) if !vec.is_empty() && !vec[0].is_empty() => Some(vec[0][0] as char),
            _ => None,
        })
        .unwrap_or('E');

    let mut lat = parse_gps_rational(lat_val)?;
    let mut lon = parse_gps_rational(lon_val)?;

    if lat_ref == 'S' { lat = -lat; }
    if lon_ref == 'W' { lon = -lon; }

    Some((lat, lon))
}

pub fn read_exif_data(path: &Path) -> Option<exif::Exif> {
    std::fs::File::open(path)
        .ok()
        .and_then(|file| {
            let mut bufreader = BufReader::new(file);
            Reader::new().read_from_container(&mut bufreader).ok()
        })
}
