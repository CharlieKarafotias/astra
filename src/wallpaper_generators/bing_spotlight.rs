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
pub fn generate_bing_spotlight(verbose: bool) -> Result<AstraImage, WallpaperGeneratorError> {
    if verbose {
        println!("Fetching today's Bing Spotlight wallpaper...");
    }
    // Credit to Spotlight Downloader project for API reference
    // https://github.com/ORelio/Spotlight-Downloader/blob/master/SpotlightAPI.md
    let res = reqwest::blocking::get("https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&bcnt=1&country=US&locale=en-US&fmt=json")
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .json::<SpotlightResponse>()
        .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;

    if res.batchrsp.items.len() == 0 {
        return Err(WallpaperGeneratorError::ImageGenerationError(
            "No images found in response".to_string(),
        ));
    }
    if verbose {
        println!("Received response with image URL");
    }

    let image_info: ImageInfo = serde_json::from_str(&res.batchrsp.items[0].item)
        .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;
    let image_url = image_info.ad.landscape_image.asset;

    if verbose {
        println!("Downloading image...");
    }
    let image = reqwest::blocking::get(image_url)
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .bytes()
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .to_vec();
    if verbose {
        println!("Image downloaded successfully");
    }

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
