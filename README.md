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

Use the configuration file to specify how often wallpapers should update,
which generators to use, and more. If no configuration file exists **OR** 
the configuration file is an empty object `{}`, the `astra` command will 
randomly select a wallpaper generator and generate a new image.

| Key                                 | Type    | Description                                                                                                                                                                                                   | Regex Format / Enum Options                                                                                                                                             | Example                                                                      | Default Behavior If Key is Not Included                                                                           |
|-------------------------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------|
| auto_clean                          | string  | If defined, will clean any image stored in astra's wallpaper folder older than the specified time.                                                                                                            | ^\d+[smhdwMy]$                                                                                                                                                          | 1w                                                                           | Never runs unless user calls clean command.                                                                       |
| frequency                           | string  | Controls the frequency at which automatic wallpaper updates occur. Supported units are: seconds(s), minutes(m), hours(h), days(d), weeks(w), months(M), and years(y).                                         | ^\d+[smhdwMy]$                                                                                                                                                          | 1d                                                                           | Always updates the wallpaper when astra command runs.                                                             |
| generators                          | array   | List of which wallpaper generators to use when `astra` command is called.                                                                                                                                     | julia \| solid \| spotlight                                                                                                                                             | ["spotlight"]                                                                | All generators are available; one is chosen at random.                                                            |
| julia_gen                           | object  | Controls settings used for the julia generator.                                                                                                                                                               | N/A                                                                                                                                                                     | see julia_gen.{key} below                                                    | Generates an image with random theme and complex number.                                                          |
| julia_gen.appearance                | string  | Controls if dark/light mode is preferred (Auto is system setting).                                                                                                                                            | Auto \| Light \| Dark                                                                                                                                                   | "Auto"                                                                       | Auto (appearance set by OS)                                                                                       |
| julia_gen.complex_numbers           | array   | List of complex numbers to use. If multiple are provided, one will be chosen at random.                                                                                                                       | ^\[-?\d*\.\d+, -?\d*\.\d+\]$                                                                                                                                            | [[0.28, 0.008], [-0.4, 0.6]]                                                 | Uses one of the pre-defined complex numbers provided by julia_gen.                                                |
| julia_gen.starting_sample_threshold | number  | The starting color intensity required for the sampling algorithm to save a sampled point. A higher number normally results in more color variety.                                                             | A number between 0 and 255                                                                                                                                              | 200                                                                          | 200                                                                                                               |
| julia_gen.respect_color_themes      | boolean | If true, the themes defined by the user will be respected when generating a julia set.                                                                                                                        | true \| false                                                                                                                                                           | true                                                                         | false                                                                                                             |
| solid_gen                           | object  | Controls settings used for the solid generator.                                                                                                                                                               | N/A                                                                                                                                                                     | see solid_gen.{key} below                                                    | Generates an image with a random rgb color.                                                                       |
| solid_gen.preferred_default_colors  | array   | List of preferred default colors. If multiple are provided, one will be chosen one at random.                                                                                                                 | [See the full list of colors here](https://github.com/CharlieKarafotias/astra/blob/5b0b318796f22fe3445ee3acca3928129c805841/src/wallpaper_generators/solid_color.rs#L8) | ["White", "Lime"]                                                            | If neither default or rgb colors are defined, then a random color is used.                                        |
| solid_gen.preferred_rgb_colors      | array   | List of preferred rgb colors. If multiple are provided, one will be chosen at random.                                                                                                                         | Any three numbers between 0 and 255                                                                                                                                     | [[196, 71, 70], [0, 51, 0]]                                                  | If neither default or rgb colors are defined, then a random color is used.                                        |
| solid_gen.respect_color_themes      | boolean | If true, the themes defined by the user will be respected when generating a solid color image.                                                                                                                | true \| false                                                                                                                                                           | true                                                                         | false                                                                                                             |
| spotlight_gen                       | object  | Controls settings used for the spotlight generator.                                                                                                                                                           | N/A                                                                                                                                                                     | see spotlight_gen.{key} below                                                | Generates image using the country key `US` and the locale key `en-US`.                                            |
| spotlight_gen.country               | string  | Controls which country to use as bing spotlight provides different images based on country.                                                                                                                   | [All 2-character country codes are listed in this wikipedia page](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)                                                     | "CN"                                                                         | "US"                                                                                                              |
| spotlight_gen.locale                | string  | Controls the region subtag to use (if supported by the country). From testing, this isn't always required.                                                                                                    | [language code](https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes)-[country code](https://en.wikipedia.org/wiki/ISO_3166-1)                                  | "en-GB"                                                                      | "en-US"                                                                                                           |
| spotlight_gen.respect_color_themes  | boolean | If true, the themes defined by the user will attempt to be respected. The algorithm pulls 4 images and compares the average color values of each image and finds closest matching to one of the users themes. | true \| false                                                                                                                                                           | true                                                                         | false                                                                                                             |
| themes                              | array   | Contains theme objects (see below) which will be used by generators that have the `respect_color_themes` property set to true.                                                                                | N/A                                                                                                                                                                     | see themes.{key} below for theme object schema `[{ name: "my theme", ... }]` | Uses the generators default setup when generating image.                                                          |
| theme.name                          | string  | The custom theme's name.                                                                                                                                                                                      | ^\w[\w\s]*$                                                                                                                                                             | "My new theme"                                                               | The name is required for a theme to be valid.                                                                     |
| theme.colors                        | array   | List of rgb colors a generator will use to make image matching the theme.                                                                                                                                     | Array of arrays containing three numbers between 0 and 255                                                                                                              | [[0, 0, 0], [255, 255, 255]]                                                 | The list of rgb colors is required for the theme to be valid                                                      |
| theme.dark_mode_colors              | array   | List of rgb colors a generator will use when dark mode context is detected.                                                                                                                                   | Array of arrays containing three numbers between 0 and 255                                                                                                              | [[0, 0, 0], [255, 255, 255]]                                                 | If excluded, no dark mode is supported and theme.colors will be used instead for both light & dark mode contexts. |

💡 More keys and customization options may be added in future releases. Feel free to suggest ideas or contribute!

#### Configuration Notes (macOS)

When making updates to the `frequency` key in your configuration file, you must:
1. Completely remove the `frequency` key from config file
2. Run `astra` command. This will unload the plist file behind the scenes
3. Ope configuration file and update re-add `frequency` key to the value desired. 

## 🤝 Contributing

Contributions are welcome! Reach out to [Charlie Karafotias](https://github.com/CharlieKarafotias) for how to contribute.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Julia Set Wikipedia Page](https://en.wikipedia.org/wiki/Julia_set) for providing background information & mathematical equations
- [ORelio's Splotlight-Downloader project](https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md) for the detailed Spotlight API documentation

### Developed by [Charlie Karafotias](https://github.com/CharlieKarafotias)

---
