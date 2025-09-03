use super::super::config::Config;
use super::utils::{AstraImage, WallpaperGeneratorError};
use serde::Deserialize;

/// Generates a wallpaper from the Bing Spotlight API. The API provides a
/// photo of the day, which is used as the wallpaper (same as Windows 11 Spotlight).
/// The image is downloaded from the URL and saved to the desktop wallpaper
/// folder with a name of the form "spotlight_<unix_timestamp>.png"
/// (if save and update are true).
///
/// All credit goes to Spotlight Downloader project for the helpful documentation on the API used
/// [Spotlight Downloader project](https://github.com/ORelio/Spotlight-Downloader).
/// The API is queried with the following parameters:
///  - `placement=88000820`
///  - `bcnt=1`
///  - `country=US`
///  - `locale=en-US`
///  - `fmt=json`
///
/// # Return & Errors
///
/// This function returns a `Result` containing a `PathBuf` to the saved
/// image on success, or a `WallpaperGeneratorError` on failure. The
/// `WallpaperGeneratorError` enum contains the following variants:
///
/// * `ImageGenerationError`: The image failed to download, or the image
///   failed to parse.
/// * `NetworkError`: The API request failed.
/// * `ParseError`: The JSON response from the API failed to parse.
pub fn generate_bing_spotlight(config: &Config) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating Bing Spotlight...");

    if config.respect_user_config {
        config.print_if_verbose("User config detected with spotlight_gen options...");
    }

    // TODO v1.1.0 - respect color theme here
    // let respect_theme = crate::respect_user_config_or_default!(config, spotlight_gen, respect_color_themes, { Ok(false) })?;

    // Pull out config fields and inject to URL if exist
    let country = crate::respect_user_config_or_default!(config, spotlight_gen, country, {
        Ok("US".to_string())
    })?;
    let locale = crate::respect_user_config_or_default!(config, spotlight_gen, locale, {
        Ok("en-US".to_string())
    })?;

    // Build URL
    let mut url = String::from(
        "https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&fmt=json&bcnt=1",
    );
    url += format!("&country={country}").as_str();
    url += format!("&locale={locale}").as_str();

    config.print_if_verbose("Fetching today's Bing Spotlight wallpaper...");
    let res = reqwest::blocking::get(url)
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .json::<SpotlightResponse>()
        .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;

    if res.batchrsp.items.len() == 0 {
        return Err(WallpaperGeneratorError::ImageGenerationError(
            "No images found in response".to_string(),
        ));
    }
    config.print_if_verbose("Received response with image URL");

    let image_info: ImageInfo = serde_json::from_str(&res.batchrsp.items[0].item)
        .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;
    let image_url = image_info.ad.landscape_image.asset;

    config.print_if_verbose("Downloading image...");

    let image = reqwest::blocking::get(image_url)
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .bytes()
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .to_vec();
    config.print_if_verbose("Image downloaded successfully");

    let loaded_image = image::load_from_memory(&image)
        .map_err(|e| WallpaperGeneratorError::ImageGenerationError(e.to_string()))?;

    Ok(loaded_image.to_rgb8())
}

// Request and response structs
#[derive(Deserialize)]
struct SpotlightResponse {
    batchrsp: ResponsePayload,
}

#[derive(Deserialize)]
struct ResponsePayload {
    items: Vec<Item>,
}

#[derive(Deserialize)]
struct Item {
    item: String,
}

#[derive(Deserialize)]
struct ImageInfo {
    ad: AdInfo,
}

#[derive(Deserialize)]
struct AdInfo {
    #[serde(rename = "landscapeImage")]
    landscape_image: LandscapeImage,
}

#[derive(Deserialize)]
struct LandscapeImage {
    asset: String,
}
