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
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".nameforge_cache.json"))
    }

    pub fn load() -> Self {
        let cache = Self::get_cache_file_path()
            .filter(|path| path.exists())
            .and_then(|path| File::open(&path).ok())
            .map(BufReader::new)
            .and_then(|reader| serde_json::from_reader::<_, GPSCache>(reader).ok());
            
        match cache {
            Some(loaded_cache) => {
                println!("{}  {}{}", "💾".bright_green(), "Loaded GPS cache with ".bright_green(), format!("{} entries", loaded_cache.cache.len()).bright_white().bold());
                loaded_cache
            }
            None => GPSCache::new()
        }
    }

    pub fn save(&self) {
        let result = Self::get_cache_file_path()
            .and_then(|path| File::create(&path).ok())
            .map(BufWriter::new)
            .and_then(|writer| serde_json::to_writer_pretty(writer, &self).ok());
            
        match result {
            Some(_) => println!("{}  {}{}", "💾".bright_green(), "Saved GPS cache with ".bright_green(), format!("{} entries", self.cache.len()).bright_white().bold()),
            None => eprintln!("{} {}", "❌".bright_red(), "Failed to save GPS cache".bright_red())
        }
    }

    pub fn get(&self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, value: String) {
        self.cache.insert(key, value);
    }
}