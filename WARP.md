# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

Nameforge is a Rust CLI tool that renames JPEG images based on their context - specifically combining timestamp information from EXIF data with location data derived from GPS coordinates. The tool uses reverse geocoding via OpenStreetMap's Nominatim API to convert GPS coordinates into human-readable place names.

## Development Commands

### Building and Running
```bash
# Build the project
cargo build

# Build for release
cargo build --release

# Check code without building
cargo check

# Run the tool (after building)
cargo run -- --input /path/to/photos

# Run with dry-run mode (shows what would be renamed without making changes)
cargo run -- --input /path/to/photos --dry-run

# Install locally for system-wide use
cargo install --path .
```

### Testing and Quality
```bash
# Run tests (when tests are added)
cargo test

# Run specific test
cargo test test_name

# Check code formatting
cargo fmt --check

# Format code
cargo fmt

# Run clippy for linting
cargo clippy

# Run clippy with all targets
cargo clippy --all-targets
```

### Development Workflow
```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Show dependency tree
cargo tree

# Check for security vulnerabilities
cargo audit  # (requires cargo-audit)
```

## Architecture Overview

### Core Components

**Main Entry Point (`main.rs`)**
- Uses `clap` for command-line argument parsing
- Accepts `--input` (required path) and `--dry-run` (optional boolean) flags
- Delegates processing to the library module

**Library Module (`lib.rs`)**
- Contains all business logic for image processing and renaming
- Key functions:
  - `process_folder()`: Main orchestrator that processes all JPEG files in a directory
  - `build_new_name()`: Core naming logic that extracts EXIF data and constructs new filenames
  - `gps_to_place()`: Reverse geocoding function with caching
  - `get_date_string()`: Date extraction with fallback to file modification time
  - `unique_filename()`: Ensures no filename conflicts by appending counters

### Data Flow

1. **File Discovery**: Scan input directory for JPEG files (.jpg extension)
2. **EXIF Extraction**: Read EXIF metadata using `kamadak-exif` crate
3. **Date Processing**: Extract DateTimeOriginal or fall back to file modification time
4. **GPS Processing**: Parse GPS coordinates and reference directions (N/S, E/W)
5. **Reverse Geocoding**: Convert coordinates to place names via Nominatim API (with caching)
6. **Filename Generation**: Combine date and place: `YYYY-MM-DD_HH-MM-SS_PlaceName.jpg`
7. **Conflict Resolution**: Add numeric suffix if filename already exists
8. **File Operation**: Rename file or show dry-run preview

### Key Dependencies

- **clap**: Command-line argument parsing with derive macros
- **kamadak-exif**: EXIF metadata reading from JPEG files
- **reqwest**: HTTP client for Nominatim API calls (blocking mode)
- **chrono**: Date/time parsing and formatting
- **serde**: JSON deserialization for API responses

### Design Patterns

**Caching Strategy**: GPS coordinate lookups are cached using a HashMap with rounded coordinates as keys to avoid redundant API calls for nearby locations.

**Error Handling**: Uses Option types extensively with fallback mechanisms:
- EXIF date extraction falls back to file modification time
- Missing GPS data results in "NoGPS" in filename
- Failed API calls result in "UnknownPlace" in filename

**File Safety**: Implements unique filename generation to prevent overwrites by appending incrementing numbers to duplicate names.

## Test Data

The `testphotos/` directory contains sample JPEG files for testing the renaming functionality.