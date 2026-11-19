# Astra 🌌

Astra is a powerful daily wallpaper generator that brings stunning, high-quality wallpapers to your desktop.
With support for Windows, macOS, and Linux, Astra makes it easy to keep your desktop fresh with beautiful, automatically
updating wallpapers.

## ✨ Features

- 🖼️ Automatically update your desktop wallpaper daily
- 🌐 Fetch wallpapers from various online sources
- 🎨 Customize wallpaper styles and categories
- 🧠 Generate original wallpapers locally with SDXL Turbo — pure Rust, no Python
- ⚡ Lightweight and fast
- 🎯 Cross-platform support (Windows, macOS, Linux)

## 🚀 Installation

### Prerequisites
- [Git](https://git-scm.com/downloads)
- [Rust](https://www.rust-lang.org/tools/install)

### Linux
```bash
# Clone the repository
git clone https://github.com/CharlieKarafotias/astra.git
cd astra

# Build the project
cargo build --release

# Move the Astra binary to preferred location (e.g., /opt, /usr/local/bin) 
# Note: You may need to run this command with sudo
sudo cp target/release/astra /usr/local/bin

# Test astra (you may need to restart your terminal)
astra -V

# For more information, see the help menu
astra --help
```

### macOS

_Utilize my homebrew tap to download and manage astra for you_
```bash
brew tap charliekarafotias/tools
brew trust charliekarafotias/tools
brew install charliekarafotias/tools/astra
```

_If you prefer to build from source_
```bash
# Clone the repository
git clone https://github.com/CharlieKarafotias/astra.git
cd astra

# Build the project
cargo build --release

# Move the Astra binary to preferred location (e.g., /usr/local/bin)
sudo cp target/release/astra /usr/local/bin

# Test astra (you may need to restart your terminal)
astra -V

# For more information, see the help menu
astra --help
```

### Windows
```powershell
# Clone the repository
git clone https://github.com/CharlieKarafotias/astra.git
cd astra

# Build the project
cargo build --release

# Move the Astra binary to preferred location (e.g., C:\Program Files\astra\astra.exe) 
# Note: You may need to run this command in an Administator terminal
Copy-Item target\release\astra.exe C:\Program Files\astra

# Test astra (you may need to restart your terminal)
astra -V

# For more information, see the help menu
astra --help
```

## 🛠️ Usage

### Basic Commands

```bash
# Sets a new random wallpaper using any one of astra's generators
astra

# Sets a new wallpaper using the spotlight generator
astra generate spotlight

# Sets a new wallpaper using the julia generator (Julia Set art)
astra generate julia

# Generates an original wallpaper locally with SDXL Turbo (needs a GPU build)
astra generate sdxl --prompt "Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text"

# Cleans up wallpapers saved to the wallpaper directory
astra clean

# View help
astra --help
```

### 🧠 Local AI wallpapers (SDXL Turbo)

Astra can generate original wallpapers on your machine with
[`stabilityai/sdxl-turbo`](https://huggingface.co/stabilityai/sdxl-turbo) and
[Hugging Face Candle](https://github.com/huggingface/candle). This path is pure
Rust: no Python, no `diffusers`, no sidecar, and no network access beyond the
Hugging Face Hub model download.

#### Build for your hardware

The default `cargo build --release` is a CPU build so CI and machines without a
GPU still compile. Diffusion on a CPU is possible but slow and memory hungry.

```bash
# macOS (Apple Silicon)
cargo run --release --features metal -- generate sdxl --prompt "..."

# Nvidia
cargo run --release --features cuda,cudnn -- generate sdxl --prompt "..."
```

`metal` and `cuda` must not be enabled in the same build.

#### First run

The first run downloads the SDXL Turbo weights into the standard Hugging Face
cache (`~/.cache/huggingface/hub`, or `$HF_HOME/hub`). They are shared with the
`hf` CLI, so `hf cache ls` and `astra models list` see the same files.

```bash
# Pre-warm the cache without generating anything
astra models download

# Check what is cached
astra models list

# Validate args, prompt and device without downloading or running diffusion
astra generate sdxl --dry-run --prompt "a misty alpine lake at sunrise"

# Generate and set the wallpaper
astra generate sdxl \
  --prompt "Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text" \
  --steps 4 --seed 42 --output wallpaper-santorini.png

# Same prompt, but quicker: ~0.6 MP canvas and 3 steps
astra generate sdxl --fast \
  --prompt "Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text"
```

Without `--width`/`--height`, the dimensions are fitted from your screen
resolution: the aspect ratio is preserved, the canvas is kept around SDXL's
native ~1 megapixel, and both sides are rounded to multiples of 8. Pass
`--width 1024 --height 1024` (or any multiple of 8 up to 2048) to override it.

`--fast` fits the same screen to ~0.6 megapixels and uses 3 steps instead of 4.
It roughly halves the run time at the cost of some fine detail. An explicit
`--steps`, `--width` or `--height` always wins over the preset.

Without `--prompt`, Astra builds one from `sdxl_gen.characteristics` (or a
shipped default set). See [docs/config.md](./docs/config.md#sdxl-turbo-generator-sdxl_gen)
for the config block.

A run is four denoising steps by default (`--steps 4`), which is what SDXL Turbo
is distilled for. On a 16 GiB M1 at 1344x768 expect roughly 1.5 minutes: about
40 s of sampling plus a 35-40 s VAE decode. The decoder runs on the CPU because
Candle's Metal decoder exceeds the GPU working set at that size; it is bounded,
not fast. `--fast` (1000x624, 3 steps) takes about 60 s, and smaller explicit
canvases (`--width 1024 --height 576`) cut both halves almost proportionally.

The generated image is also saved to Astra's wallpaper folder and set as the
desktop wallpaper, exactly like the other generators. Use `--no-save --no-update`
to only write `--output`.

#### Where models are cached

- Default: `~/.cache/huggingface/hub`
- Override: `export HF_HOME=/path/to/huggingface`
- Authentication: `export HF_TOKEN=...` if the Hub asks for one. Tokens are never
  written to disk or logs by Astra.

Re-running a command reuses the cache and does not download the weights again.

#### Troubleshooting

- **"SDXL Turbo cannot run safely"** — the machine has no Metal/CUDA build or
  less than 16 GiB of memory. Build with the right `--features`, or pass
  `--allow-cpu` (expect it to be slow).
- **Out of memory** — try a smaller `--width`/`--height` such as `1024x1024`, or
  use `--allow-cpu` to move the work to system memory.
- **Download failures** — check your connection, set `HF_TOKEN` if the Hub is
  rate limiting you, then run `astra models download`.
- **`metal` vs `cuda`** — enable the features matching your machine. Enabling a
  GPU feature on the wrong platform will not produce a usable device; Astra falls
  back to the CPU only when `--allow-cpu` is given.

### macOS notes

macOS sets each generated wallpaper by its unique file name, just like other
platforms. macOS also keeps every image set this way in System Settings →
Wallpaper → "Your Photos", even after the file is deleted. When auto-clean
(`auto_clean` in the config) or `astra clean` deletes old wallpapers, Astra also
removes those stale Astra entries and their cached thumbnails from the gallery.
Astra then restarts WallpaperAgent and the image wallpaper extension so the
gallery updates immediately. If System Settings is open at the time, it can
briefly re-show removed entries until it refreshes; the next clean removes them
again. Only entries pointing at deleted Astra wallpapers are touched — your other
photos and the current wallpaper are left alone.

### ⚙️ Configuration File

Astra supports an optional JSON configuration file that allows you to customize
its behavior without needing to pass flags or subcommands every time. This file
can be placed at a standard location depending on your OS (see below) and will
be automatically loaded when the `astra` command runs.

#### Standard Location For File Based On OS

| OS      | Standard Location                                                          |
|---------|----------------------------------------------------------------------------|
| Linux   | \$XDG_DATA_HOME/astra/config.json                                          |
|         | \$HOME/.config/astra/config.json                                           |
| macOS   | \$HOME/Library/Application Support/dev.CharlieKarafotias.Astra/config.json |
| Windows | {FOLDERID_RoamingAppData}\CharlieKarafotias\Astra\config\config.json       |

_Astra uses the [`directories`](https://lib.rs/crates/directories) crate for this functionality_

#### Supported Configuration Keys

Check out [docs/config.md](./docs/config.md) file for all supported keys. Run
`astra config` to install the JSON Schema and add its `$schema` reference to
your config, enabling editor autocomplete and validation. Use
`astra config --schema` to print the schema instead.

## 🤝 Contributing

Contributions are welcome! Reach out to [Charlie Karafotias](https://github.com/CharlieKarafotias) for how to contribute.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Julia Set Wikipedia Page](https://en.wikipedia.org/wiki/Julia_set) for providing background information & mathematical equations
- [ORelio's Splotlight-Downloader project](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md) for the detailed Spotlight API documentation

### Developed by [Charlie Karafotias](https://github.com/CharlieKarafotias)

---
