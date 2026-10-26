# Astra — Specification

> Living document. Astra is currently v1.3.1. This spec describes the product
> as implemented today; known TODOs are marked as such.

## 1. Overview

Astra is a cross-platform (Linux, macOS, Windows) command-line tool that
generates desktop wallpapers and sets them automatically — either procedurally
rendered (Julia-set fractals, solid colors) or fetched from online sources
(Bing Spotlight, NASA Astronomy Picture of the Day).

Target user: anyone who wants fresh, automatically-updating desktop wallpapers
with minimal effort.

## 2. Features

### 2.1 Wallpaper generators

| Generator   | Description                                                        |
|-------------|--------------------------------------------------------------------|
| `julia`     | Rayon-parallelized Julia-set fractals; 13 built-in complex constants, 256-step gradient color maps |
| `spotlight` | Bing Spotlight daily photo                                         |
| `nasa_apod` | NASA Astronomy Picture of the Day (HTML scrape, no API key needed)  |
| `solid`     | 20 named colors, random, or custom RGB                             |

### 2.2 Themes

10 built-in themes (NeonDreams, AuroraGlow, CyberSunset, MysticForest, RetroPop,
OceanBreeze, GalaxyVoyage, FireIce, CandyCrush, SunlitMeadow) plus user-defined
themes via config. Generators can opt into `respect_color_themes`.

### 2.3 Auto mode & scheduling

- Bare `astra` honors the full user config: random generator pick, `auto_clean`,
  `frequency`-based scheduling.
- Linux: systemd user service + timer (`~/.config/systemd/user/`).
- macOS: launchd plist (`~/Library/LaunchAgents/`), 10-minute tick granularity.
- Windows: `schtasks` task; each run briefly flashes a console window (expected).

### 2.4 CLI

- `astra` — auto mode (config-driven).
- `astra generate <generator> [--no-save] [--no-update]` — explicit generation;
  bypasses global config (frequency, generator list) but honors per-generator options.
  - `nasa-apod [--date YYMMDD]`
  - `solid <color <name>|random|rgb <r> <g> <b>>`
- `astra clean [--older-than <freq>] [--directory]` — delete cached wallpapers.
- `astra config [--open]` — create/print config path; `--open` opens it in `$EDITOR`.
- `astra generate-completions <shell>` — shell completions.
- Global `-v/--verbose`.

### 2.5 Configuration

JSON config at OS-standard paths (`directories` crate, qualifier `dev`,
org `CharlieKarafotias`, app `Astra`):

- Linux: `$XDG_DATA_HOME/astra/config.json` or `~/.config/astra/config.json`
- macOS: `~/Library/Application Support/dev.CharlieKarafotias.Astra/config.json`
- Windows: `%AppData%\CharlieKarafotias\Astra\config\config.json`

All keys optional; missing/empty config = defaults. Notable keys: `auto_clean`,
`frequency` (`^\d+[smhdwMy]$`), `generators`, per-generator `*_gen` blocks,
`themes`. Known limitation: any JSON parse error discards the entire config
with a warning (TODO: partial processing).

## 3. Architecture

- `src/main.rs` — parse CLI, build `Config`, dispatch.
- `src/cli.rs` — clap derive definitions.
- `src/configuration/` — `Config`/`UserConfig` (serde JSON), `frequency.rs`,
  per-generator config, `theme.rs`.
- `src/wallpaper_generators/` — each generator returns `AstraImage`
  (`ImageBuffer<Rgb<u8>>`); `utils.rs` handles saving, deletion, color maps.
- `src/themes/` — `ColorTheme`, `ThemeSelector`, default themes.
- `src/os_implementations/` — `cfg(target_os)`-gated `linux`/`macos`/`windows`
  modules (dark-mode detection, resolution, `update_wallpaper`, schedulers).

Wallpapers are saved to `<data_dir>/Wallpapers/` as `<prefix>_<unix_timestamp>.png`
on all platforms. `julia`/`solid` use dark-mode-aware palettes via per-OS
detection. Linux wallpaper support is GNOME-only. On macOS, auto-clean and
`astra clean` also remove deleted Astra wallpapers from the System Settings
wallpaper gallery ("Your Photos"), including their cached thumbnails. The
WallpaperAgent and image wallpaper extension processes are restarted so the
gallery refreshes immediately.

The gallery is backed by two undocumented stores that both have to be pruned:
WallpaperAgent's view-model cache
(`$TMPDIR/../C/com.apple.wallpaper.agent/com.apple.wallpaper.view-model-cache/extension-com.apple.wallpaper.extension.image-desktop`)
and the image extension's history
(`~/Library/Containers/com.apple.wallpaper.extension.image/Data/Library/Preferences/com.apple.wallpaper.extension.image.plist`,
`ChoiceRequests.ImageFiles`). Thumbnails live under
`~/Library/Containers/com.apple.wallpaper.extension.image/Data/Library/Caches`.
Astra prunes only entries that point at missing files inside its own wallpapers
directory; everything else (including bookmark data and timestamps of kept
entries) is left untouched.

## 4. Releases

- `Cargo.toml` `[package] version` is the single source of truth.
- Push to `main` → `.github/workflows/release-and-update.yml`: tags `v<VERSION>`,
  creates a GitHub release, and dispatches the Homebrew formula update
  (`charliekarafotias/homebrew-tools`). Install via
  `brew tap charliekarafotias/tools && brew install charliekarafotias/tools/astra`.

## 5. Known Limitations / TODOs

- NASA APOD relies on HTML scraping (fragile if markup changes); Spotlight uses
  an undocumented endpoint.
- `solid` generator reads `julia_gen.appearance` for dark mode (known bug, TODO).
- Non-GNOME Linux distros unsupported for wallpaper setting.
- macOS scheduler granularity is effectively 10 minutes; Windows sub-minute
  frequencies clamp to 1 minute.
