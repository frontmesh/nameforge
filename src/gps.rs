use reqwest::blocking::Client;
use serde::Deserialize;
use crate::cache::GPSCache;
use colored::*;

#[derive(Deserialize)]
struct NominatimResponse {
    display_name: String,
}

pub fn to_key(lat: f64, lon: f64) -> (i64, i64) {
    ((lat * 1e6) as i64, (lon * 1e6) as i64)
}

pub fn to_cache_key(lat: f64, lon: f64) -> String {
    let (lat_i, lon_i) = to_key(lat, lon);
    format!("{}_{}", lat_i, lon_i)
}

pub fn gps_to_place(lat: f64, lon: f64, cache: &mut GPSCache) -> (Option<String>, bool) {
    let key = to_cache_key(lat, lon);
    if let Some(place) = cache.get(&key) {
        return (Some(place.clone()), false); // Found in cache, no update
    }
    
    println!("{}  {}({}, {})...", "🌍".bright_blue(), "Resolving GPS coordinates ".bright_blue(), lat.to_string().bright_white(), lon.to_string().bright_white());
    let client = Client::new();
    let url = format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=10&addressdetails=0", 
        lat, lon
    );
    let resp = client.get(&url)
        .header("User-Agent", "nameforge/1.0")
        .send().ok();
    
    if let Some(resp) = resp {
        if let Ok(nominatim) = resp.json::<NominatimResponse>() {
            let place = nominatim.display_name.split(',').next().unwrap_or("UnknownPlace").trim().replace(' ', "_");
            cache.insert(key, place.clone());
            return (Some(place), true); // New lookup, cache updated
        }
    }
    
    // API call failed, cache a fallback
    let fallback = "UnknownPlace".to_string();
    cache.insert(key, fallback.clone());
    (Some(fallback), true)
}