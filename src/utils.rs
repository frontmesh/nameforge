use std::path::Path;

pub fn create_date_folder_path(base_folder: &Path, new_filename: &str) -> std::path::PathBuf {
    // Extract date from the new filename (format: YYYY-MM-DD_HH-MM-SS_...)
    let date_part = new_filename.split('_').next().unwrap_or("unknown-date");
    
    // Create the date folder path
    let date_folder = base_folder.join(date_part);
    date_folder.join(new_filename)
}

pub fn unique_filename(folder: &Path, base_name: &str, ext: &str) -> Option<String> {
    // Check if base_name already ends with the extension to avoid double extensions
    let filename_base = if base_name.ends_with(&format!(".{}", ext)) {
        base_name.to_string()
    } else {
        format!("{}.{}", base_name, ext)
    };
    
    let mut filename = filename_base.clone();
    let mut counter = 1;
    while folder.join(&filename).exists() {
        if base_name.ends_with(&format!(".{}", ext)) {
            // If base_name already has extension, insert counter before extension
            let stem = base_name.trim_end_matches(&format!(".{}", ext));
            filename = format!("{}_{}.{}", stem, counter, ext);
        } else {
            filename = format!("{}_{}.{}", base_name, counter, ext);
        }
        counter += 1;
    }
    Some(filename)
}
