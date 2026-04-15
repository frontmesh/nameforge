use std::path::Path;

pub fn create_date_folder_path(
    base_folder: &Path,
    date_part: &str,
    new_filename: &str,
) -> std::path::PathBuf {
    base_folder.join(date_part).join(new_filename)
}

pub fn sanitize_filename_fragment(input: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_was_separator = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch);
            previous_was_separator = false;
            continue;
        }

        if !previous_was_separator && !sanitized.is_empty() {
            sanitized.push('_');
            previous_was_separator = true;
        }
    }

    while sanitized.ends_with('_') {
        sanitized.pop();
    }

    sanitized
}

/// Helper function to normalize filename with extension
fn normalize_filename_with_extension(base_name: &str, ext: &str) -> String {
    let ext_suffix = format!(".{}", ext);
    if base_name.ends_with(&ext_suffix) {
        base_name.to_string()
    } else {
        format!("{}.{}", base_name, ext)
    }
}

/// Helper function to generate filename with counter
fn generate_filename_with_counter(base_name: &str, ext: &str, counter: u32) -> String {
    let ext_suffix = format!(".{}", ext);
    if base_name.ends_with(&ext_suffix) {
        let stem = base_name.trim_end_matches(&ext_suffix);
        format!("{}_{}.{}", stem, counter, ext)
    } else {
        format!("{}_{}.{}", base_name, counter, ext)
    }
}

fn is_available_path(candidate_path: &Path, original_path: Option<&Path>) -> bool {
    match original_path {
        Some(original_path) if candidate_path == original_path => true,
        _ => !candidate_path.exists(),
    }
}

pub fn unique_filename(
    folder: &Path,
    original_path: Option<&Path>,
    base_name: &str,
    ext: &str,
) -> Option<String> {
    let initial_filename = normalize_filename_with_extension(base_name, ext);

    // Generate sequence of potential filenames until we find one that doesn't exist
    std::iter::once(initial_filename.clone())
        .chain((1..).map(|counter| generate_filename_with_counter(base_name, ext, counter)))
        .find(|filename| is_available_path(&folder.join(filename), original_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn create_temp_dir() -> std::path::PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("nameforge-test-{}", timestamp));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    #[test]
    fn sanitize_filename_fragment_normalizes_separators() {
        assert_eq!(
            sanitize_filename_fragment("Trip to/Paris (Final)!"),
            "Trip_to_Paris_Final"
        );
    }

    #[test]
    fn unique_filename_allows_the_current_path() {
        let temp_dir = create_temp_dir();
        let existing_path = temp_dir.join("2024-05-01_video.mp4");
        fs::write(&existing_path, b"video").unwrap();

        let filename = unique_filename(&temp_dir, Some(&existing_path), "2024-05-01_video", "mp4");

        assert_eq!(filename.as_deref(), Some("2024-05-01_video.mp4"));

        fs::remove_file(existing_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }

    #[test]
    fn unique_filename_uses_a_counter_for_collisions() {
        let temp_dir = create_temp_dir();
        let existing_path = temp_dir.join("2024-05-01_video.mp4");
        fs::write(&existing_path, b"video").unwrap();

        let filename = unique_filename(&temp_dir, None, "2024-05-01_video", "mp4");

        assert_eq!(filename.as_deref(), Some("2024-05-01_video_1.mp4"));

        fs::remove_file(existing_path).unwrap();
        fs::remove_dir(temp_dir).unwrap();
    }
}
