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

/// Helper function to build Nominatim API URL
fn build_nominatim_url(lat: f64, lon: f64) -> String {
    format!(
        "https://nominatim.openstreetmap.org/reverse?format=json&lat={}&lon={}&zoom=10&addressdetails=0",
        lat, lon
    )
}

/// Helper function to extract place name from Nominatim response
fn extract_place_name(display_name: &str) -> String {
    display_name
        .split(',')
        .next()
        .unwrap_or("UnknownPlace")
        .trim()
        .replace(' ', "_")
}

/// Helper function to perform API request and extract place
fn fetch_place_from_api(lat: f64, lon: f64) -> Option<String> {
    println!(
        "{}  {}({}, {})...",
        "🌍".bright_blue(),
        "Resolving GPS coordinates ".bright_blue(),
        lat.to_string().bright_white(),
        lon.to_string().bright_white()
    );
    
    let client = Client::new();
    let url = build_nominatim_url(lat, lon);
    
    client
        .get(&url)
        .header("User-Agent", "nameforge/1.0")
        .send()
        .ok()
        .and_then(|resp| resp.json::<NominatimResponse>().ok())
        .map(|nominatim| extract_place_name(&nominatim.display_name))
}

pub fn gps_to_place(lat: f64, lon: f64, cache: &mut GPSCache) -> (Option<String>, bool) {
    let key = to_cache_key(lat, lon);
    
    // Check cache first
    if let Some(place) = cache.get(&key) {
        return (Some(place.clone()), false);
    }
    
    // Try API request, fallback to "UnknownPlace" if it fails
    let place = fetch_place_from_api(lat, lon)
        .unwrap_or_else(|| "UnknownPlace".to_string());
    
    // Cache the result (whether successful or fallback)
    cache.insert(key, place.clone());
    (Some(place), true)
}
