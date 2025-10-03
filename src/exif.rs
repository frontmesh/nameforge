use std::{fs, io::BufReader, path::Path};
use chrono::{NaiveDateTime, DateTime, Local};
use exif::{Reader, Tag, In, Field, Value};
use colored::*;

pub fn parse_gps_rational(field: Option<&Field>) -> Option<f64> {
    let field = field?;
    if let Value::Rational(ref vec) = field.value {
        if vec.len() >= 3 {
            let deg = vec[0].to_f64();
            let min = vec[1].to_f64();
            let sec = vec[2].to_f64();
            return Some(deg + min / 60.0 + sec / 3600.0);
        }
    }
    None
}

pub fn get_date_string(path: &Path, exif_opt: &Option<exif::Exif>) -> Option<String> {
    if let Some(exif) = exif_opt {
        if let Some(field) = exif.get_field(Tag::DateTimeOriginal, In::PRIMARY) {
            let date_str = field.display_value().with_unit(exif).to_string();
            if let Ok(date) = NaiveDateTime::parse_from_str(&date_str, "%Y:%m:%d %H:%M:%S") {
                return Some(date.format("%Y-%m-%d_%H-%M-%S").to_string());
            } else {
                eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "Failed to parse EXIF date for ".bright_yellow(), path.display().to_string().bright_white(), "falling back to file modified time".bright_yellow());
            }
        } else {
            eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "No EXIF DateTimeOriginal for ".bright_yellow(), path.display().to_string().bright_white(), "falling back to file modified time".bright_yellow());
        }
    } else {
        eprintln!("{} {}{}  {}", "⚠️".bright_yellow(), "No EXIF data for ".bright_yellow(), path.display().to_string().bright_white(), "falling back to file modified time".bright_yellow());
    }

    let metadata = fs::metadata(path).ok()?;
    let modified = metadata.modified().ok()?;
    let dt: DateTime<Local> = modified.into();
    Some(dt.format("%Y-%m-%d_%H-%M-%S").to_string())
}

pub fn extract_gps_coordinates(exif: &exif::Exif) -> Option<(f64, f64)> {
    let lat_val = exif.get_field(Tag::GPSLatitude, In::PRIMARY);
    let lon_val = exif.get_field(Tag::GPSLongitude, In::PRIMARY);
    let lat_ref = match exif.get_field(Tag::GPSLatitudeRef, In::PRIMARY)
        .map(|f| &f.value) {
        Some(Value::Ascii(vec)) if !vec.is_empty() && !vec[0].is_empty() => vec[0][0] as char,
        _ => 'N',
    };
    let lon_ref = match exif.get_field(Tag::GPSLongitudeRef, In::PRIMARY)
        .map(|f| &f.value) {
        Some(Value::Ascii(vec)) if !vec.is_empty() && !vec[0].is_empty() => vec[0][0] as char,
        _ => 'E',
    };

    let mut lat = parse_gps_rational(lat_val)?;
    let mut lon = parse_gps_rational(lon_val)?;

    if lat_ref == 'S' {
        lat = -lat;
    }
    if lon_ref == 'W' {
        lon = -lon;
    }

    Some((lat, lon))
}

pub fn read_exif_data(path: &Path) -> Option<exif::Exif> {
    let file = std::fs::File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);
    Reader::new().read_from_container(&mut bufreader).ok()
}