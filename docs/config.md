# Astra Configuration Reference

Astra is a powerful daily wallpaper generator that updates your desktop background automatically using various built-in generators and sources.
This document describes every configuration option available to Astra.

---

## Configuration Overview

Astra reads its configuration from an optional **JSON** file.  
If no configuration file exists, or if the file is an empty object (`{}`), Astra will randomly select a wallpaper generator each time it runs.

### File Format

All configuration files must be written in **JSON**.

```json
{
  "frequency": "1d",
  "generators": ["spotlight"]
}
```

---

## Configuration File Locations

Depending on the operating system, Astra searches for the configuration file in these locations:

| OS      | Path                                                                        |
| ------- | --------------------------------------------------------------------------- |
| Linux   | `$XDG_DATA_HOME/astra/config.json` or `$HOME/.config/astra/config.json`     |
| macOS   | `$HOME/Library/Application Support/dev.CharlieKarafotias.Astra/config.json` |
| Windows | `{FOLDERID_RoamingAppData}\CharlieKarafotias\Astra\config\config.json`      |

_Astra uses the [`directories`](https://lib.rs/crates/directories) crate to determine these standard paths._

---

## General Settings

### `auto_clean`

If defined, Astra will automatically remove any wallpapers in its cache directory that are older than the specified time.
Units include seconds (`s`), minutes (`m`), hours (`h`), days (`d`), weeks (`w`), months (`M`), and years (`y`).

**Type:** string  
**Format:** `^\d+[smhdwMy]$`  
**Example:** `"1w"`  
**Default:** Never runs unless the user explicitly calls `astra clean`.

---

### `frequency`

Controls how often Astra automatically updates your wallpaper. For example, when setting `1d` this means your wallpaper will change every day.
Units include seconds (`s`), minutes (`m`), hours (`h`), days (`d`), weeks (`w`), months (`M`), and years (`y`).

**Type:** string  
**Format:** `^\d+[smhdwMy]$`  
**Example:** `"1d"`  
**Default:** Automatic updates are disabled; must run `astra` to update wallpaper.

#### OS Specific Notes:

##### macOS

On macOS, the frequency setting is implemented using `launchd`. Internally, `astra` creates a launchd job that runs every 10 minutes.
Each time it runs, it checks whether your configured frequency duration has elapsed.
If it has, `astra` runs normally and applies the rest of your configuration.

Because of this design, any frequency below 10 minutes is treated as 10 minutes, and any frequency that is not aligned to a 10-minute interval will still be evaluated on the next 10-minute mark.

##### Windows

When adjusting the frequency key on Windows, be aware that each automatic run of `astra` will briefly show a flashing Command Prompt window. This is expected behavior, as `astra` runs under the current user account.

Windows also has a few scheduling limitations to consider:
	1.	Frequencies between 1s and 60s are rounded up to 1m due to `schtasks` limitations.
	2.	Frequencies expressed in seconds that exceed one minute (e.g., 90s) are converted to minutes and rounded (e.g., 90s becomes 1m).
	3.	Frequencies longer than one year are reduced to 12M, because `schtasks` cannot schedule intervals greater than one year.

---

### `generators`

A list of wallpaper generators to choose from when `astra` runs. If multiple generators are listed, Astra will select one at random.

**Type:** array  
**Allowed values:** `["julia", "solid", "spotlight"]`  
**Example:** `["spotlight", "solid"]`  
**Default:** All available generators are used; one is chosen randomly.

---

## Julia Generator (`julia_gen`)

Controls specific to the Julia fractal generator.

### `julia_gen.appearance`

Preferred color appearance mode.

**Type:** string  
**Options:** `Auto` | `Light` | `Dark`  
**Example:** `"Auto"`  
**Default:** `"Auto"` (matches system appearance)

---

### `julia_gen.complex_numbers`

List of complex numbers used to generate fractals.  
If multiple are provided, one will be selected randomly.

**Type:** array of arrays `[real, imaginary]`  
**Format:** `[-?\d*\.\d+, -?\d*\.\d+]`  
**Example:** `[[0.28, 0.008], [-0.4, 0.6]]`  
**Default:** Uses predefined internal complex numbers.

---

### `julia_gen.starting_sample_threshold`

Defines the starting color intensity threshold for sampling points.  
Higher values generally increase color variety.

**Type:** number  
**Range:** `0–255`  
**Example:** `200`  
**Default:** `200`

---

### `julia_gen.respect_color_themes`

If `true`, the Julia generator will attempt to use user-defined color themes.

**Type:** boolean  
**Example:** `true`  
**Default:** `false`

---

## Solid Generator (`solid_gen`)

Controls the generator that creates solid-color wallpapers.

### `solid_gen.preferred_default_colors`

A list of named colors from Astra’s predefined palette.  
If multiple are listed, one will be selected randomly.

**Type:** array of strings  
**Example:** `["White", "Lime"]`  
**Default:** Random color if no colors are provided.  
**Reference:** [Full color list](https://github.com/CharlieKarafotias/astra/blob/main/src/wallpaper_generators/solid_color.rs#L8)

---

### `solid_gen.preferred_rgb_colors`

List of RGB colors to choose from.  
Each value must be an array of three numbers between 0 and 255.

**Type:** array of arrays `[r, g, b]`  
**Example:** `[[196, 71, 70], [0, 51, 0]]`  
**Default:** Random color if not defined.

---

### `solid_gen.respect_color_themes`

When `true`, Astra will adjust solid colors to match user-defined themes.

**Type:** boolean  
**Example:** `true`  
**Default:** `false`

---

## Spotlight Generator (`spotlight_gen`)

Controls wallpapers fetched from Microsoft’s Bing Spotlight service.

### `spotlight_gen.country`

Specifies which country’s spotlight feed to use.

**Type:** string  
**Format:** ISO-3166-1 alpha-2 country code  
**Example:** `"CN"`  
**Default:** `"US"`

---

### `spotlight_gen.locale`

Specifies a locale variant, if supported for the given country.

**Type:** string  
**Format:** `[language code]-[country code]` (ISO-639 + ISO-3166)  
**Example:** `"en-GB"`  
**Default:** `"en-US"`

---

### `spotlight_gen.respect_color_themes`

When `true`, Astra analyzes candidate spotlight images and selects one matching your color themes.

**Type:** boolean  
**Example:** `true`  
**Default:** `false`

---

## Themes

Custom color themes allow generators to create images that match a consistent aesthetic.

### `themes`

Contains a list of theme objects. Each theme defines a name and color palettes.

**Type:** array of objects  
**Example:**

```json
{
  "themes": [
    {
      "name": "My new theme",
      "colors": [
        [0, 0, 0],
        [255, 255, 255]
      ],
      "dark_mode_colors": [
        [0, 0, 0],
        [255, 255, 255]
      ]
    }
  ]
}
```

**Default:** Generators use built-in colors when no themes are provided.

---

### `theme.name`

A theme’s display name.

**Type:** string  
**Format:** `^\w[\w\s]*$`  
**Example:** `"My new theme"`  
**Default:** Required for the theme to be valid.

---

### `theme.colors`

List of RGB color arrays used for light mode or general contexts.

**Type:** array of arrays `[r, g, b]`  
**Example:** `[[0, 0, 0], [255, 255, 255]]`  
**Default:** Required for the theme to be valid.

---

### `theme.dark_mode_colors`

Optional array of RGB color arrays used when dark mode is active.

**Type:** array of arrays `[r, g, b]`  
**Example:** `[[0, 0, 0], [255, 255, 255]]`  
**Default:** Falls back to `theme.colors` if omitted.

---

