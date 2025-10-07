use std::path::Path;

/// Helper function to extract date part from filename
fn extract_date_part(filename: &str) -> &str {
    filename.split('_').next().unwrap_or("unknown-date")
}

pub fn create_date_folder_path(base_folder: &Path, new_filename: &str) -> std::path::PathBuf {
    let date_part = extract_date_part(new_filename);
    base_folder.join(date_part).join(new_filename)
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

pub fn unique_filename(folder: &Path, base_name: &str, ext: &str) -> Option<String> {
    let initial_filename = normalize_filename_with_extension(base_name, ext);
    
    // Generate sequence of potential filenames until we find one that doesn't exist
    std::iter::once(initial_filename.clone())
        .chain((1..).map(|counter| generate_filename_with_counter(base_name, ext, counter)))
        .find(|filename| !folder.join(filename).exists())
}
