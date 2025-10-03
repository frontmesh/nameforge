use std::path::Path;

pub fn create_date_folder_path(base_folder: &Path, new_filename: &str) -> std::path::PathBuf {
    // Extract date from the new filename (format: YYYY-MM-DD_HH-MM-SS_...)
    let date_part = new_filename.split('_').next().unwrap_or("unknown-date");
    
    // Create the date folder path
    let date_folder = base_folder.join(date_part);
    date_folder.join(new_filename)
}

pub fn unique_filename(folder: &Path, base_name: &str, ext: &str) -> Option<String> {
    let mut filename = format!("{}.{}", base_name, ext);
    let mut counter = 1;
    while folder.join(&filename).exists() {
        filename = format!("{}_{}.{}", base_name, counter, ext);
        counter += 1;
    }
    Some(filename)
}