# 📸 NameForge (`nf`)

**Intelligent photo renaming tool with AI content analysis, GPS location resolution, and smart organization.**

NameForge transforms your photo collection from generic filenames like `IMG_1234.jpg` into meaningful, contextual names like `2023-08-31_cozy_livingroom.jpg` using EXIF metadata, GPS coordinates, and AI-powered content analysis.

**Quick Start:** Use the convenient `nf` alias for faster commands!

## ✨ Features

- 🤖 **AI Content Analysis** - Uses Ollama vision models to analyze photo content
- 🌍 **GPS Location Resolution** - Converts GPS coordinates to readable place names
- 📅 **Date-based Organization** - Automatically sorts photos into date folders
- 💾 **Smart Caching** - Persistent GPS cache to avoid redundant API calls
- 🎨 **Customizable Naming** - Multiple case formats and character limits
- 🌈 **Beautiful UI** - Colorful, emoji-rich terminal interface
- 🏃 **Dry Run Mode** - Preview changes before applying them
- 📊 **Batch Processing** - Handle entire photo collections efficiently

## 🚀 Installation

### 📦 Option 1: Download Pre-built Binary (Recommended)

Download the latest binary for your platform from [GitHub Releases](https://github.com/frontmesh/nameforge/releases):

**macOS (Apple Silicon)**:
```bash
curl -L https://github.com/frontmesh/nameforge/releases/latest/download/nameforge-aarch64-apple-darwin.tar.gz | tar xz
sudo mv nameforge /usr/local/bin/
sudo mv nf /usr/local/bin/
```

**macOS (Intel)**:
```bash
curl -L https://github.com/frontmesh/nameforge/releases/latest/download/nameforge-x86_64-apple-darwin.tar.gz | tar xz
sudo mv nameforge /usr/local/bin/
sudo mv nf /usr/local/bin/
```

**Linux (x86_64)**:
```bash
curl -L https://github.com/frontmesh/nameforge/releases/latest/download/nameforge-x86_64-unknown-linux-gnu.tar.gz | tar xz
sudo mv nameforge /usr/local/bin/
sudo mv nf /usr/local/bin/
```

**Windows**: Download `nameforge-x86_64-pc-windows-msvc.zip` from releases and extract to your PATH.

### 🍺 Option 2: Homebrew (macOS/Linux)

```bash
# Install from Homebrew (coming soon to homebrew-core)
brew install nameforge
```

### 🦀 Option 3: Cargo (Build from Source)

**Prerequisites**: Rust 1.70 or later

```bash
# Install from crates.io
cargo install nameforge

# Or install from GitHub (latest)
cargo install --git https://github.com/frontmesh/nameforge
```

### 🐧 Option 4: Package Managers

**Arch Linux (AUR)**:
```bash
# Coming soon
yay -S nameforge
```

**Windows (Scoop)**:
```bash
# Coming soon
scoop install nameforge
```

### Prerequisites (Optional)

- **Ollama** (for AI content analysis) - Optional but recommended

#### Installing Ollama

**macOS**:
```bash
brew install ollama
brew services start ollama
ollama pull llava-llama3:latest
```

**Linux**:
```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llava-llama3:latest
```

**Windows**: Download from [ollama.ai](https://ollama.ai/download)

## 📖 Usage

### Basic Usage

```bash
# Rename photos using GPS data (dry run) - uses date-only format by default
nf --input /path/to/photos --dry-run

# Actually rename the files
nf --input /path/to/photos

# Organize into date folders
nf --input /path/to/photos --organize-by-date

# Use full timestamps if needed
nf --input /path/to/photos --full-timestamp --dry-run
```

### AI Content Analysis

```bash
# Enable AI content analysis
nf --input /path/to/photos --ai-content --dry-run

# Customize AI parameters
nf --input /path/to/photos --ai-content \
  --ai-model llava:13b \
  --ai-max-chars 15 \
  --ai-case snake_case \
  --ai-language English
```

### Advanced Examples

```bash
# Full-featured example (date-only is default)
nf --input ~/Pictures/Vacation2023 \
  --ai-content \
  --organize-by-date \
  --ai-case snake_case \
  --ai-max-chars 20 \
  --dry-run

# Process with different AI model and full timestamps
nf --input /path/to/photos \
  --ai-content \
  --ai-model llava-llama3 \
  --ai-case camelCase \
  --full-timestamp
```

## 🎛️ Options

| Option | Description | Default |
|--------|-------------|---------|
| `--input` | Path to photo directory | Required |
| `--dry-run` | Preview changes without applying | `false` |
| `--organize-by-date` | Create date-based folder structure | `false` |
| `--full-timestamp` | Use full timestamp instead of date-only | `false` |
| `--ai-content` | Enable AI content analysis | `false` |
| `--ai-model` | Ollama model to use | `llava:13b` |
| `--ai-max-chars` | Maximum characters for AI filename | `20` |
| `--ai-case` | Case format (lowercase, uppercase, snake_case, camelCase) | `lowercase` |
| `--ai-language` | Language for AI-generated names | `English` |

## 🎯 How It Works

1. **📁 Scan Directory** - Finds all JPEG files in the specified directory
2. **📊 Extract EXIF** - Reads metadata including date and GPS coordinates
3. **🌍 Resolve Location** - Converts GPS coordinates to place names via OpenStreetMap
4. **🤖 AI Analysis** - (Optional) Analyzes image content for descriptive naming
5. **📝 Generate Names** - Creates meaningful filenames with timestamps and context
6. **📂 Organize** - (Optional) Sorts into date-based folder structure

## 📋 Filename Format

### Default Format (Date-Only)
```
YYYY-MM-DD_LocationOrContent.jpg
```

### Full Timestamp Format (with --full-timestamp)
```
YYYY-MM-DD_HH-MM-SS_LocationOrContent.jpg
```

### Examples
- `2023-08-31_Paris.jpg` (GPS-based, default)
- `2023-08-31_cozy_livingroom.jpg` (AI-based, default)
- `2023-08-31_13-12-20_Paris.jpg` (GPS-based, with --full-timestamp)
- `2023-09-15_sunset_beach.jpg` (AI with snake_case, default)

### With Date Organization
```
photos/
├── 2023-08-31/
│   ├── 2023-08-31_13-12-20_Paris.jpg
│   └── 2023-08-31_21-48-55_cozy_livingroom.jpg
├── 2023-09-01/
│   └── 2023-09-01_14-22-45_London.jpg
└── 2023-09-02/
    └── 2023-09-02_16-30-12_sunset_beach.jpg
```

## 🛠️ Configuration

### Supported AI Models
- `llava:13b` (default, balanced accuracy/speed)
- `llava-llama3` (faster, good accuracy)
- `llava:34b` (higher accuracy, slower)
- Any other Ollama vision model

### Case Formats
- `lowercase` - `beach_sunset`
- `uppercase` - `BEACH_SUNSET`
- `snake_case` - `beach_sunset`
- `camelCase` - `beachSunset`

### Languages
Supports any language - AI will generate filenames in the specified language:
- `English` (default)
- `Spanish`
- `French`
- `German`
- etc.

## 💾 Caching

NameForge automatically caches GPS lookups in `~/.nameforge_cache.json` to:
- Avoid redundant API calls
- Speed up subsequent runs
- Work offline for previously seen locations

## 🎨 Visual Interface

NameForge features a beautiful, colorful terminal interface with:

- 📸 **Configuration Display** - Shows all active settings
- 📷 **Processing Indicators** - Visual progress through photo collection
- 🌍 **GPS Resolution** - Real-time coordinate lookup feedback
- 🤖 **AI Analysis** - Model processing status and results
- ✨ **Results** - Highlighted filename generation
- 💁 **Dry Run Preview** - Clear before → after transformations

## 🔧 Troubleshooting

### Common Issues

**Ollama Connection Failed**
```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Start Ollama service
brew services start ollama
```

**No GPS Data**
- Photos will be named with `NoGPS` instead of location
- Consider enabling `--ai-content` for better naming

**EXIF Date Parsing Failed**
- Falls back to file modification time
- Warning message will be shown in yellow

**Empty AI Response**
- Try a different model with `--ai-model`
- Check Ollama model availability: `ollama list`

## 🏗️ Architecture

NameForge is built with a modular architecture:

- `main.rs` - CLI interface and configuration display
- `lib.rs` - Main orchestration logic
- `ai.rs` - Ollama AI content analysis
- `cache.rs` - Persistent GPS caching
- `exif.rs` - EXIF metadata processing
- `gps.rs` - GPS coordinate resolution
- `utils.rs` - Utility functions

## 🤝 Contributing

Contributions are welcome! Areas for improvement:
- Additional AI model support
- More case format options
- Extended EXIF metadata handling
- Performance optimizations

## 📄 License

MIT License - feel free to use and modify as needed.

## 🙏 Acknowledgments

- **OpenStreetMap Nominatim** for GPS resolution
- **Ollama** for local AI inference
- **Rust Community** for excellent crates and tools