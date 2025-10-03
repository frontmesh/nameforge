use std::{collections::HashMap, fs::File, io::{BufReader, BufWriter}, path::PathBuf};
use serde::{Deserialize, Serialize};
use serde_json;
use colored::*;

#[derive(Serialize, Deserialize)]
pub struct GPSCache {
    cache: HashMap<String, String>,
}

impl GPSCache {
    pub fn new() -> Self {
        GPSCache {
            cache: HashMap::new(),
        }
    }

    fn get_cache_file_path() -> Option<PathBuf> {
        let home_dir = std::env::var("HOME").ok()?;
        Some(PathBuf::from(home_dir).join(".nameforge_cache.json"))
    }

    pub fn load() -> Self {
        if let Some(cache_path) = Self::get_cache_file_path() {
            if cache_path.exists() {
                if let Ok(file) = File::open(&cache_path) {
                    if let Ok(cache) = serde_json::from_reader::<_, GPSCache>(BufReader::new(file)) {
                        println!("{}  {}{}", "💾".bright_green(), "Loaded GPS cache with ".bright_green(), format!("{} entries", cache.cache.len()).bright_white().bold());
                        return cache;
                    }
                }
            }
        }
        GPSCache::new()
    }

    pub fn save(&self) {
        if let Some(cache_path) = Self::get_cache_file_path() {
            if let Ok(file) = File::create(&cache_path) {
                if let Err(e) = serde_json::to_writer_pretty(BufWriter::new(file), &self) {
                    eprintln!("{} {}{}", "❌".bright_red(), "Failed to save GPS cache: ".bright_red(), e.to_string().bright_white());
                } else {
                    println!("{}  {}{}", "💾".bright_green(), "Saved GPS cache with ".bright_green(), format!("{} entries", self.cache.len()).bright_white().bold());
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.cache.insert(key, value);
    }
}