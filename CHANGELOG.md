# Changelog

All notable changes to this project should be tracked in this file.

## [Unreleased]

### Added

- Recursive media discovery for nested working copies.
- Mixed-library support for common video formats alongside existing image formats.
- A project-level changelog so notable changes are tracked outside commit messages.
- Tests covering filename sanitization and collision handling for already-renamed files.

### Changed

- Renaming now treats the project as a mixed photo/video tool instead of an image-only tool.
- Photo files continue to use EXIF, GPS, and optional AI naming, while video files use filesystem dates and filename fallbacks.
- Files without usable GPS or AI results now fall back to sanitized original stems when they are meaningful, instead of defaulting to placeholders like `NoGPS`.
- Collision checks now run against the real destination folder, which fixes date-folder moves and avoids forcing `_1` onto files that are already correctly named.
- CLI help text and README examples now describe recursive scanning and mixed-media behavior.
- Processing options are now passed through typed structs instead of long argument lists, which keeps the rename pipeline clippy-clean.

### Fixed

- GPS cache entries are only written for successful lookups instead of persisting failed `UnknownPlace` results.
- CI now uses a maintained Rust audit action instead of the broken `actions-rs/audit` reference.
- Clippy failures from redundant imports, literal formatting, and manual error inspection were removed.
