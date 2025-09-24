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

    // Pull out config fields and inject to URL if exist
    let country = crate::respect_user_config_or_default!(config, spotlight_gen, country, {
        Ok("US".to_string())
    })?;
    let locale = crate::respect_user_config_or_default!(config, spotlight_gen, locale, {
        Ok("en-US".to_string())
    })?;

    // Pull 2-4 images and find one that matches the closest to the average color of the theme
    let respect_theme =
        crate::respect_user_config_or_default!(config, spotlight_gen, respect_color_themes, {
            Ok(false)
        })?;

    let download_links = get_image_download_urls(
        &config,
        APIParams {
            // TODO: v1.1.0 - this could be 2-4, maybe count could be config option as 4 could be slow due to blocking calls
            count: if respect_theme { 2 } else { 1 },
            country: &country,
            locale: &locale,
        },
    )?;

    let mut selected_image = Vec::new();
    for download_link in download_links {
        let image = download_image_to_memory(&config, &download_link)?;
        // TODO: v1.1.0 - implement respect_color_themes by comparing average color of image to themes available
        // From here, choose the best image matching any of the user_config themes
        // If no good images are found then just return the first image

        // TODO: remove me after implementing above, doing this for commit
        selected_image = image;
        break;
    }

    let loaded_image = image::load_from_memory(selected_image.as_slice())
        .map_err(|e| WallpaperGeneratorError::ImageGenerationError(e.to_string()))?;

    Ok(loaded_image.to_rgb8())
}

fn download_image_to_memory(
    config: &Config,
    url: &str,
) -> Result<Vec<u8>, WallpaperGeneratorError> {
    config.print_if_verbose(format!("Downloading image from {}", url).as_str());
    let image = reqwest::blocking::get(url)
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .bytes()
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .to_vec();
    config.print_if_verbose("Image downloaded successfully");
    Ok(image)
}

// TODO: v1.1.0 - add func comments
fn get_image_download_urls(
    config: &Config,
    params: APIParams,
) -> Result<Vec<String>, WallpaperGeneratorError> {
    let url = build_url(params);
    config.print_if_verbose("Fetching download URLs for spotlight wallpaper(s)...");
    let res = reqwest::blocking::get(url)
        .map_err(|e| WallpaperGeneratorError::NetworkError(e.to_string()))?
        .json::<SpotlightResponse>()
        .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;
    if res.batchrsp.items.len() == 0 {
        return Err(WallpaperGeneratorError::ImageGenerationError(
            "No download URLs found in response".to_string(),
        ));
    }
    config.print_if_verbose(
        format!(
            "Received response with {} image download URLs",
            res.batchrsp.items.len()
        )
        .as_str(),
    );

    let mut urls: Vec<String> = Vec::new();
    for element in res.batchrsp.items {
        let image_info: ImageInfo = serde_json::from_str(&element.item)
            .map_err(|e| WallpaperGeneratorError::ParseError(e.to_string()))?;
        urls.push(image_info.ad.landscape_image.asset);
    }
    Ok(urls)
}

// TODO: v1.1.0 - add func comments
// TODO: v1.1.0 - add tests
fn build_url(params: APIParams) -> String {
    format!(
        "https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&fmt=json&bcnt={count}&country={country}&locale={locale}",
        count = params.count,
        country = params.country,
        locale = params.locale,
    )
}

struct APIParams<'a> {
    count: u8,
    country: &'a str,
    locale: &'a str,
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
