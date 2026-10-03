# Astra Agents Guide

## Project Overview

**Astra** is a Rust CLI tool that automatically updates desktop wallpapers using multiple generators:
- **Spotlight** (Bing daily images)
- **Julia Set** (procedural fractals)
- **Solid Color** (predefined or custom RGB)
- **NASA APOD** (Astronomy Picture of the Day)

**Version:** 1.3.1  
**License:** MIT  
**Author:** Charlie Karafotias

---

## Key Directories

```
src/
├── main.rs                           # Entry point, CLI parsing
├── cli.rs                            # Command-line interface definitions
├── constants.rs                      # Magic constants, file paths
├── themes/                           # Color theme handling
│   ├── mod.rs                        # Theme module
│   ├── theme_selector.rs             # Applies themes to generators
│   ├── color_theme.rs                # Color palette definitions
│   └── default_themes.rs             # Built-in themes
├── configuration/                     # Config parsing & validation
│   ├── mod.rs
│   ├── config.rs                     # Main config struct
│   ├── user_config.rs                # Merges config + CLI args
│   ├── frequency.rs                  # Frequency parsing (1d, 1h, etc.)
│   ├── theme.rs                      # Theme validation
│   └── generators/                   # Per-generator config
│       ├── solid.rs                  # Solid color options
│       ├── julia.rs                  # Julia fractal options
│       ├── spotlight.rs              # Bing spotlight options
│       └── nasa_apod.rs              # NASA APOD options
├── wallpaper_generators/              # Image generation logic
│   ├── mod.rs                        # Generator registry
│   ├── utils.rs                      # Shared utilities
│   ├── solid_color.rs                # Solid color generator
│   ├── julia.rs                      # Julia set fractal generator
│   ├── bing_spotlight.rs             # Bing API integration
│   └── nasa_apod.rs                  # NASA API integration
└── os_implementations/               # OS-specific wallpaper setting
    ├── macos/                        # macOS integration
    ├── linux/                        # Linux/systemd integration
    └── windows/                      # Windows Task Scheduler
```

---

## Build & Run

```bash
# Build in release mode
cargo build --release

# Run with version flag
cargo run -- --version

# Run with help
cargo run -- --help
```

---

## Configuration File

Location (OS-specific):
- **Linux:** `$HOME/.config/astra/config.json`
- **macOS:** `$HOME/Library/Application Support/dev.CharlieKarafotias.Astra/config.json`
- **Windows:** `{FOLDERID_RoamingAppData}\CharlieKarafotias\Astra\config\config.json`

### Core Keys

```json
{
  "frequency": "1d",           // Update interval (1s-1y)
  "generators": ["spotlight"], // Available generators (random selection)
  "auto_clean": "1w",          // Auto-cleanup old wallpapers
  "themes": [...]              // Custom color themes
}
```

### Frequency Format
- `1d` = 1 day
- `1h` = 1 hour
- `1w` = 1 week
- `30m` = 30 minutes

### Generator Options

#### Julia Generator
```json
{
  "julia_gen": {
    "appearance": "Auto",       // "Auto" | "Light" | "Dark"
    "complex_numbers": [[0.28, 0.008], [-0.4, 0.6]],
    "starting_sample_threshold": 200,
    "respect_color_themes": true
  }
}
```

#### Solid Generator
```json
{
  "solid_gen": {
    "preferred_default_colors": ["White", "Lime"],
    "preferred_rgb_colors": [[196, 71, 70], [0, 51, 0]],
    "respect_color_themes": true
  }
}
```

#### Spotlight Generator
```json
{
  "spotlight_gen": {
    "country": "US",            // ISO-3166-1 alpha-2
    "locale": "en-US",          // ISO-639 + ISO-3166
    "respect_color_themes": true
  }
}
```

#### NASA APOD Generator
```json
{
  "nasa_apod_gen": {
    "date_from": "260423",      // yymmdd format
    "date_to": "260423"
  }
}
```

---

## Dependencies (Cargo.toml)

**Core:**
- `chrono` - Time handling
- `clap` / `clap_complete` - CLI parsing
- `directories` - Standard config/data paths
- `image` - Image processing (PNG)
- `num-complex` - Complex number math for Julia sets
- `rand` - Random selection
- `rayon` - Parallel processing
- `reqwest` - HTTP requests (Bing, NASA APIs)
- `serde` / `serde_json` - JSON config

**Platform-specific:**
- **Windows:** `windows` crate (Win32 API)
- **macOS:** `objc2-*` crates (Objective-C bindings)

---

## Important Pitfalls & Notes

### macOS Frequency Implementation
- Uses `launchd` internally
- Creates a job that runs every 10 minutes
- Checks elapsed time against configured frequency
- Frequencies < 10min are treated as 10min
- Not aligned to exact intervals (next 10-min mark)

### Windows Frequency Limitations
- 1s-60s → rounded up to 1m
- >60s in seconds → converted to minutes
- >1 year → capped at 12M

### Wallpaper Storage Locations
- **Linux:** `~/.local/share/astra/wallpapers`
- **macOS:** `~/Library/Application Support/dev.CharlieKarafotias.Astra/wallpapers`
- **Windows:** `{FOLDERID_RoamingAppData}\CharlieKarafotias\Astra\data\wallpapers`

### macOS Wallpaper Hack (v1.2.0)
To force wallpaper refresh on macOS, use only 2 filenames: `astra_1` or `astra_2`. The OS detects path changes and refreshes wallpaper + cache.

---

## CLI Commands

```bash
astra                          # Random wallpaper from configured generators
astra generate <generator>    # Force specific generator
astra generate spotlight      # Bing Spotlight
astra generate julia          # Julia fractal
astra clean                    # Remove old wallpapers
astra sync                     # Synchronize wallpapers across all monitors (new in v1.3.1)
astra --help                   # Show help
astra -V                       # Show version
```

---

## Development Notes

### Testing Generators
Each generator has its own module in `wallpaper_generators/`. Test them individually by calling `cargo run -- generate <name>`.

### Color Themes
Themes are applied when `respect_color_themes: true` in generator config. Themes define RGB palettes for light/dark modes.

### API Rate Limits
- **Bing Spotlight:** No known rate limits
- **NASA APOD:** Cache results; NASA API may have limits

### Build Artifacts
- Debug: `target/debug/astra`
- Release: `target/release/astra`
- Ignored: `target/`, `*.png`, `*.jpg`

---

## Release Notes

See [RELEASENOTES.md](./RELEASENOTES.md) for changelog. Latest: v1.3.1 with sync functionality.

---

## Contributing

Contact: Charlie Karafotias (@CharlieKarafotias)

---

## External References

- [Julia Set Wikipedia](https://en.wikipedia.org/wiki/Julia_set)
- [Bing Spotlight API](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md)
- [Rust image crate](https://docs.rs/image)
- [Num-complex crate](https://docs.rs/num-complex)
