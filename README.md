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

TODO: update me with the right paths
| Operating System | Default Locaton         | Override by setting variable |
|------------------|-------------------------|------------------------------|
| Linux            | /home/\<user>/.config    | $XDG_CONFIG_HOME             |
| macOS            | /Users/\<user>/.config   | $XDG_CONFIG_HOME             |
| Windows          | C:\Users\\\<user>\\.config | $XDG_CONFIG_HOME             |

_Astra uses the [`cross-xdg`](https://lib.rs/crates/cross-xdg) crate for this functionality_

#### Supported Configuration Keys

Use the configuration file to specify how often wallpapers should update,
which generators to use, and more. If no configuration file exists **OR** 
the configuration file is an empty object `{}`, the `astra` command will 
randomly select a wallpaper generator and generate a new image.

| Key        | Type   | Description                          | Regex Format            | Example       | Default Behavior                                                |
|------------|--------|--------------------------------------|-------------------------|---------------|-----------------------------------------------------------------|
| frequency  | String | Controls automatic wallpaper updates | ^\d+[s \| m \| h \| d]$ | 1d            | Will always update wallpaper when astra command runs            |
| generators | array  | List of wallpaper generators to use  | N/A                     | ["spotlight"] | All generators are available; astra command picks one at random |

💡 More keys and customization options may be added in future releases. Feel free to suggest ideas or contribute!


## 🤝 Contributing

Contributions are welcome! Reach out to [Charlie Karafotias](https://github.com/CharlieKarafotias) for how to contribute.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ORelio's Splotlight-Downloader project](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md) for the detailed Spotlight API documentation

### Developed by [Charlie Karafotias](https://github.com/CharlieKarafotias)

---
