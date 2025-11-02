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

Controls how often Astra automatically updates your wallpaper.  
Units include seconds (`s`), minutes (`m`), hours (`h`), days (`d`), weeks (`w`), months (`M`), and years (`y`).

**Type:** string  
**Format:** `^\d+[smhdwMy]$`  
**Example:** `"1d"`  
**Default:** Automatic updates are disabled; must run `astra` to update wallpaper.

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

## macOS Configuration Notes

When modifying the `frequency` key on macOS, follow these steps to ensure the system daemon reloads properly:

1. Remove the `frequency` key from your configuration file.
2. Run `astra` once to unload the background service.
3. Add `frequency` back to the file with your new desired value.

## Windows Configuration Notes

When modifying the `frequency` key on Windows, note that everytime the system runs astra automatically, a cmd window will flash. This is astra running as the user.

Further, note there are a few limitations on Windows for frequency:

1. Setting frequency between `1s` and `60s` will become `1m` on Windows due to schtasks limitation.
2. Setting frequency with to some seconds like `90s` will convert to minutes and round (this will be `1m` on Windows).
3. Setting frequency to anything above a year will change to `12m` as schtasks does not support higher than 1 year.

---
