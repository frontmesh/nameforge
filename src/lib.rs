mod ai;
mod cache;
mod exif;
mod gps;
mod utils;

use ai::get_ai_content_name;
use cache::GPSCache;
use colored::*;
use exif::{extract_gps_coordinates, get_date_string, get_file_date_string, read_exif_data};
use gps::gps_to_place;
use std::{
    fs,
    path::{Path, PathBuf},
};
use utils::{create_date_folder_path, sanitize_filename_fragment, unique_filename};

const SUPPORTED_IMAGE_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "tiff", "tif", "bmp", "webp", "heic", "heif", "raw", "cr2", "nef", "arw",
];
const SUPPORTED_VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "m4v", "avi", "mkv", "mts", "m2ts", "mpg", "mpeg", "3gp", "webm",
];
const GENERIC_CAMERA_PREFIXES: &[&str] = &[
    "img", "dsc", "mov", "mvi", "vid", "pxl", "dji", "imgp", "gopr", "gp", "mvimg",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MediaKind {
    Image,
    Video,
}

impl MediaKind {
    fn label(self) -> &'static str {
        match self {
            MediaKind::Image => "image",
            MediaKind::Video => "video",
        }
    }

    fn fallback_label(self) -> &'static str {
        match self {
            MediaKind::Image => "photo",
            MediaKind::Video => "video",
        }
    }
}

#[derive(Debug)]
struct MediaFile {
    path: PathBuf,
    kind: MediaKind,
}

#[derive(Debug)]
struct RenamePlan {
    base_name: String,
    extension: String,
    date_folder: Option<String>,
    gps_cache_updated: bool,
}

fn is_supported_extension(extension: &str, supported_extensions: &[&str]) -> bool {
    supported_extensions.contains(&extension.to_ascii_lowercase().as_str())
}

fn classify_media_kind(path: &Path) -> Option<MediaKind> {
    let extension = path.extension()?.to_str()?;

    if is_supported_extension(extension, SUPPORTED_IMAGE_EXTENSIONS) {
        return Some(MediaKind::Image);
    }

    if is_supported_extension(extension, SUPPORTED_VIDEO_EXTENSIONS) {
        return Some(MediaKind::Video);
    }

    None
}

fn is_not_resource_fork(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| !name.starts_with("._"))
        .unwrap_or(true)
}

fn collect_media_files(input_path: &Path) -> Result<Vec<MediaFile>, String> {
    if input_path.is_file() {
        return classify_media_kind(input_path)
            .filter(|_| is_not_resource_fork(input_path))
            .map(|kind| {
                vec![MediaFile {
                    path: input_path.to_path_buf(),
                    kind,
                }]
            })
            .ok_or_else(|| format!("Not a supported media file: {}", input_path.display()));
    }

    if input_path.is_dir() {
        let mut media_files = Vec::new();
        collect_media_files_from_dir(input_path, &mut media_files)?;
        return Ok(media_files);
    }

    Err(format!(
        "Input path does not exist or is not accessible: {}",
        input_path.display()
    ))
}

fn collect_media_files_from_dir(
    dir_path: &Path,
    media_files: &mut Vec<MediaFile>,
) -> Result<(), String> {
    let mut entries: Vec<_> = fs::read_dir(dir_path)
        .map_err(|error| format!("Could not open folder {}: {}", dir_path.display(), error))?
        .filter_map(Result::ok)
        .collect();

    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        if !is_not_resource_fork(&path) {
            continue;
        }

        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            collect_media_files_from_dir(&path, media_files)?;
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        if let Some(kind) = classify_media_kind(&path) {
            media_files.push(MediaFile { path, kind });
        }
    }

    Ok(())
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
    let media_files = match collect_media_files(input_path) {
        Ok(files) => files,
        Err(error) => {
            eprintln!("{} {}", "❌".bright_red(), error.bright_white());
            return;
        }
    };

    if media_files.is_empty() {
        println!(
            "{}  {}",
            "📊".bright_blue(),
            "Found 0 supported media files to process".bright_blue()
        );
        return;
    }

    let image_count = media_files
        .iter()
        .filter(|media_file| media_file.kind == MediaKind::Image)
        .count();
    let video_count = media_files.len() - image_count;

    println!(
        "{}  {}{}{}{}{}{}{}{}",
        "📊".bright_blue(),
        "Found ".bright_blue(),
        media_files.len().to_string().bright_white().bold(),
        " supported media files to process ".bright_blue(),
        "(".bright_black(),
        image_count.to_string().bright_white().bold(),
        " images, ".bright_black(),
        video_count.to_string().bright_white().bold(),
        " videos)".bright_black()
    );

    let base_folder = get_base_folder(input_path).to_path_buf();
    let mut gps_cache = GPSCache::load();
    let mut cache_updated = false;
    let mut processed_count = 0;

    for media_file in media_files {
        if max_images
            .map(|max| processed_count >= max)
            .unwrap_or(false)
        {
            println!(
                "{}  {}{}{}",
                "🎯".bright_cyan(),
                "Reached maximum file limit of ".bright_cyan(),
                max_images.unwrap().to_string().bright_white().bold(),
                ". Stopping processing.".bright_cyan()
            );
            break;
        }

        println!(
            "{}  {}{}{}",
            "📷".bright_blue(),
            "Processing ".bright_blue(),
            media_file.kind.label().bright_white().bold(),
            format!(" file: {}", media_file.path.display()).bright_blue()
        );

        if let Some(rename_plan) = build_rename_plan(
            &media_file,
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
            cache_updated |= rename_plan.gps_cache_updated;

            let target_folder = get_target_folder(
                &media_file.path,
                &base_folder,
                rename_plan.date_folder.as_deref(),
                organize_by_date,
            );
            let Some(new_name) = unique_filename(
                &target_folder,
                Some(&media_file.path),
                &rename_plan.base_name,
                &rename_plan.extension,
            ) else {
                eprintln!(
                    "{} {}{}",
                    "❌".bright_red(),
                    "Failed to generate a unique filename for ".bright_red(),
                    media_file.path.display().to_string().bright_white()
                );
                processed_count += 1;
                continue;
            };

            let new_path = get_target_path(
                &media_file.path,
                &base_folder,
                rename_plan.date_folder.as_deref(),
                &new_name,
                organize_by_date,
            );

            if media_file.path == new_path {
                print_skip_info(&media_file.path);
            } else if dry_run {
                print_dry_run_info(&media_file.path, &new_path);
            } else if let Err(error) = execute_rename(&media_file.path, &new_path) {
                eprintln!(
                    "{} {}{} {} {}{}  {}{}",
                    "❌".bright_red(),
                    "Failed to rename ".bright_red(),
                    media_file.path.display().to_string().bright_white(),
                    "→".bright_red(),
                    new_path.display().to_string().bright_white(),
                    ": ".bright_red(),
                    "Error: ".bright_red(),
                    error.to_string().bright_white()
                );
            }
        }

        processed_count += 1;
    }

    if cache_updated {
        gps_cache.save();
    }
}

fn resolve_gps_location(
    exif_opt: &Option<::exif::Exif>,
    cache: &mut GPSCache,
) -> (Option<String>, bool) {
    exif_opt
        .as_ref()
        .and_then(extract_gps_coordinates)
        .map(|(lat, lon)| gps_to_place(lat, lon, cache))
        .unwrap_or((None, false))
}

fn generate_ai_content(
    path: &Path,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
) -> Option<String> {
    get_ai_content_name(path, ai_model, ai_max_chars, ai_case, ai_language).or_else(|| {
        eprintln!(
            "{} {}{}  {}",
            "⚠️".bright_yellow(),
            "Failed to get AI content analysis for ".bright_yellow(),
            path.display().to_string().bright_white(),
            "using filename fallback".bright_yellow()
        );
        None
    })
}

fn get_media_date_string(
    media_kind: MediaKind,
    path: &Path,
    exif_opt: &Option<::exif::Exif>,
    date_only: bool,
    use_file_date: bool,
    prefer_modified: bool,
) -> Option<String> {
    if media_kind == MediaKind::Video {
        return get_file_date_string(path, date_only, prefer_modified);
    }

    get_date_string(path, exif_opt, date_only, use_file_date, prefer_modified)
}

fn resolve_content_part(
    media_file: &MediaFile,
    cache: &mut GPSCache,
    ai_content: bool,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
    exif_opt: &Option<::exif::Exif>,
) -> (String, bool) {
    if ai_content {
        return resolve_ai_content_part(media_file, ai_model, ai_max_chars, ai_case, ai_language);
    }

    if media_file.kind == MediaKind::Image {
        let (place, updated) = resolve_gps_location(exif_opt, cache);
        let content =
            place.unwrap_or_else(|| fallback_name_from_path(&media_file.path, media_file.kind));
        return (
            sanitize_content_or_fallback(&content, &media_file.path, media_file.kind),
            updated,
        );
    }

    (
        fallback_name_from_path(&media_file.path, media_file.kind),
        false,
    )
}

fn resolve_ai_content_part(
    media_file: &MediaFile,
    ai_model: &str,
    ai_max_chars: u32,
    ai_case: &str,
    ai_language: &str,
) -> (String, bool) {
    if media_file.kind == MediaKind::Video {
        eprintln!(
            "{} {}{}",
            "⚠️".bright_yellow(),
            "AI content analysis currently supports still images only; using filename fallback for "
                .bright_yellow(),
            media_file.path.display().to_string().bright_white()
        );
        return (
            fallback_name_from_path(&media_file.path, media_file.kind),
            false,
        );
    }

    let content = generate_ai_content(
        &media_file.path,
        ai_model,
        ai_max_chars,
        ai_case,
        ai_language,
    )
    .unwrap_or_else(|| fallback_name_from_path(&media_file.path, media_file.kind));

    (
        sanitize_content_or_fallback(&content, &media_file.path, media_file.kind),
        false,
    )
}

fn sanitize_content_or_fallback(content: &str, path: &Path, media_kind: MediaKind) -> String {
    let sanitized = sanitize_filename_fragment(content);
    if sanitized.is_empty() {
        return fallback_name_from_path(path, media_kind);
    }

    sanitized
}

fn fallback_name_from_path(path: &Path, media_kind: MediaKind) -> String {
    let original_stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(sanitize_filename_fragment)
        .unwrap_or_default();

    if original_stem.is_empty() || is_generic_camera_stem(&original_stem) {
        return media_kind.fallback_label().to_string();
    }

    original_stem
}

fn is_generic_camera_stem(stem: &str) -> bool {
    let compact_stem = stem
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_lowercase();

    GENERIC_CAMERA_PREFIXES.iter().any(|prefix| {
        compact_stem.starts_with(prefix)
            && compact_stem
                .chars()
                .skip(prefix.len())
                .any(|ch| ch.is_ascii_digit())
    })
}

fn create_base_filename(date_fmt: Option<String>, content_part: String) -> String {
    date_fmt
        .map(|date| format!("{}_{}", date, content_part))
        .unwrap_or(content_part)
}

fn get_base_folder(input_path: &Path) -> &Path {
    if input_path.is_dir() {
        input_path
    } else {
        input_path.parent().unwrap_or(Path::new("."))
    }
}

fn get_target_folder(
    original_path: &Path,
    base_folder: &Path,
    date_folder: Option<&str>,
    organize_by_date: bool,
) -> PathBuf {
    if organize_by_date {
        if let Some(date_folder) = date_folder {
            return base_folder.join(date_folder);
        }
    }

    original_path.parent().unwrap_or(base_folder).to_path_buf()
}

fn get_target_path(
    original_path: &Path,
    base_folder: &Path,
    date_folder: Option<&str>,
    new_name: &str,
    organize_by_date: bool,
) -> PathBuf {
    if organize_by_date {
        if let Some(date_folder) = date_folder {
            return create_date_folder_path(base_folder, date_folder, new_name);
        }
    }

    original_path.with_file_name(new_name)
}

fn print_skip_info(path: &Path) {
    println!(
        "{}  {}{}",
        "ℹ️".bright_cyan(),
        "No rename needed for ".bright_cyan(),
        path.display().to_string().bright_white()
    );
}

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

fn execute_rename(original_path: &Path, new_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = new_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|error| {
                eprintln!(
                    "{} {}{}{}  {}{}",
                    "❌".bright_red(),
                    "Failed to create directory ".bright_red(),
                    parent.display().to_string().bright_white(),
                    ": ".bright_red(),
                    "Error: ".bright_red(),
                    error.to_string().bright_white()
                );
                error
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

    fs::rename(original_path, new_path)
        .map_err(|error| Box::new(error) as Box<dyn std::error::Error>)?;

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

fn build_rename_plan(
    media_file: &MediaFile,
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
) -> Option<RenamePlan> {
    let exif_opt = if media_file.kind == MediaKind::Image {
        read_exif_data(&media_file.path)
    } else {
        None
    };
    let date_prefix = if no_date {
        None
    } else {
        get_media_date_string(
            media_file.kind,
            &media_file.path,
            &exif_opt,
            date_only,
            use_file_date,
            prefer_modified,
        )
    };
    let date_folder = get_media_date_string(
        media_file.kind,
        &media_file.path,
        &exif_opt,
        true,
        use_file_date,
        prefer_modified,
    );
    let extension = media_file.path.extension()?.to_str()?.to_string();
    let (content_part, gps_cache_updated) = resolve_content_part(
        media_file,
        cache,
        ai_content,
        ai_model,
        ai_max_chars,
        ai_case,
        ai_language,
        &exif_opt,
    );
    let base_name = create_base_filename(date_prefix, content_part);

    Some(RenamePlan {
        base_name,
        extension,
        date_folder,
        gps_cache_updated,
    })
}
