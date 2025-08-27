# Astra Release Notes

## v1.1.0

### New Features
- New configuration file (see README.md) to specify user preferences:
  - Frequency to generate new wallpaper
  - Preferred wallpaper generators
### Changes
- `astra` command functionality
  - Running `astra` will first check for configuration file. If it exists, respect settings
  - The `astra` command can be called on terminal startup. If user specifies frequency for wallpaper updates, will update if needed.
  - Fallback to existing functionality when no configuration file (randomly select wallpaper generator)
- Generated wallpapers are no longer added to the astra wallpapers folder on Desktop. Instead, find them in the following places:

| OS      | Standard Location                                                         |
|---------|---------------------------------------------------------------------------|
| Linux   | \$XDG_DATA_HOME/astra/wallpapers                                          |
|         | \$HOME/.local/share/astra/wallpapers                                      |
| macOS   | \$HOME/Library/Application Support/dev.CharlieKarafotias.Astra/wallpapers |
| Windows | {FOLDERID_RoamingAppData}\CharlieKarafotias\Astra\data\wallpapers         |

## v1.0.3

### Bug Fix
- Correct issue with automated releases
- Update readme with homebrew install directions

## v1.0.2

### Improvements
- Automated releases

## v1.0.1

### Bug Fix
- Error with tagging on release causing homebrew to pull old version

## v1.0.0

### New Features
- **Spotlight Image Support**: Astra now supports generating wallpapers using Bing's daily Spotlight images.
- **Solid Background Images**: Create wallpapers with solid colors, including pre-defined, random, and custom RGB colors.
- **Julia Set Creation**: Generate stunning fractal wallpapers with Julia Set art.

### Platform Support
- **Cross-Platform Compatibility**: Astra is compatible with Windows, macOS, and Linux, offering seamless functionality across different operating systems.

### Improvements
- Enhanced performance and reliability for wallpaper generation.
- Improved user interface for command line operations.

### Bug Fixes
- Fixed known issues related to wallpaper updates on different platforms.
- Resolved potential errors in image generation processes.

For more information, refer to the [documentation](https://github.com/CharlieKarafotias/astra).

