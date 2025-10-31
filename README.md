# Astra 🌌

Astra is a powerful daily wallpaper generator that brings stunning, high-quality wallpapers to your desktop.
With support for Windows, macOS, and Linux, Astra makes it easy to keep your desktop fresh with beautiful, automatically
updating wallpapers.

## ✨ Features

- 🖼️ Automatically update your desktop wallpaper daily
- 🌐 Fetch wallpapers from various online sources
- 🎨 Customize wallpaper styles and categories
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
brew install astra
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

# Cleans up wallpapers saved to the wallpaper directory
astra clean

# View help
astra --help
```

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

Check out [docs/config.md](./docs/config.md) file for all supported keys.

## 🤝 Contributing

Contributions are welcome! Reach out to [Charlie Karafotias](https://github.com/CharlieKarafotias) for how to contribute.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Julia Set Wikipedia Page](https://en.wikipedia.org/wiki/Julia_set) for providing background information & mathematical equations
- [ORelio's Splotlight-Downloader project](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md) for the detailed Spotlight API documentation

### Developed by [Charlie Karafotias](https://github.com/CharlieKarafotias)

---
