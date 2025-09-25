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

    // Check if user has defined color themes
    let has_user_defined_color_themes = config.themes().is_some()
        && config
            .themes()
            .expect("Failed to get themes")
            .themes()
            .len()
            > 0;

    let download_links = get_image_download_urls(
        &config,
        APIParams {
            // TODO: v1.1.0 - this could be 2-4, maybe count could be config option as 4 could be slow due to blocking calls
            count: if respect_theme && has_user_defined_color_themes {
                2
            } else {
                1
            },
            country: &country,
            locale: &locale,
        },
    )?;

    let selected_image: AstraImage = if respect_theme && has_user_defined_color_themes {
        // Loop through each download URL
        // download image
        // Convert to AstraImage
        // Compare AstraImage to user themes to find the closest theme for curr image
        // return the image closest to one of the user themes
        for link in download_links {
            let downloaded_img = download_image_to_memory(&config, &link)?;
            let loaded_img: AstraImage = image::load_from_memory(downloaded_img.as_slice())
                .map_err(|e| WallpaperGeneratorError::ImageGenerationError(e.to_string()))?
                .to_rgb8();
            // Function that tells us how far off the average color of the image is from the average color of a theme
            let distance_from_closest_theme = compare_image_to_user_theme(&config, &loaded_img);
        }
        todo!("v1.1.0 - implement me")
    } else {
        let downloaded_img = download_image_to_memory(&config, &download_links[0])?;
        image::load_from_memory(downloaded_img.as_slice())
            .map_err(|e| WallpaperGeneratorError::ImageGenerationError(e.to_string()))?
            .to_rgb8()
    };

    Ok(selected_image)
}

/// Compares the average color of an image to the average colors of each user theme.
/// Returns an integer where the lower the number is the better. The best possible match is 0.
fn compare_image_to_user_theme(config: &Config, image: &AstraImage) -> u32 {
    todo!(
        "v1.1.0 - implement me. Should find way to calculate all user themes averages once and pass ref into this. Also sum the r,g,b distances from the average colors to get a single value to compare."
    )
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
