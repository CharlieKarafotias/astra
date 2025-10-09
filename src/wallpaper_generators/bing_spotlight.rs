use super::super::configuration::Config;
use super::{
    average_color,
    utils::{AstraImage, WallpaperGeneratorError},
};
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
/// * `ImageGeneration`: The image failed to download, or the image
///   failed to parse.
/// * `Network`: The API request failed.
/// * `Parse`: The JSON response from the API failed to parse.
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
        && !config
            .themes()
            .expect("Failed to get themes")
            .themes()
            .is_empty();

    let download_links = get_image_download_urls(
        config,
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
        let user_theme_averages = compute_user_theme_averages(config)?;
        let mut best_distance: u32 = u32::MAX;
        let mut best_image: Option<AstraImage> = None;
        for link in download_links {
            let downloaded_img = download_image_to_memory(config, &link)?;
            let loaded_img: AstraImage = image::load_from_memory(downloaded_img.as_slice())
                .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?
                .to_rgb8();
            let distance_from_closest_theme = compare_image_to_user_theme_averages(
                config,
                &user_theme_averages,
                average_color(&loaded_img).0,
            );
            if distance_from_closest_theme < best_distance {
                best_distance = distance_from_closest_theme;
                best_image = Some(loaded_img);
            }
        }
        if let Some(image) = best_image {
            image
        } else {
            return Err(WallpaperGeneratorError::ImageGeneration(
                "Failed to find best image match".to_string(),
            ));
        }
    } else {
        let downloaded_img = download_image_to_memory(config, &download_links[0])?;
        image::load_from_memory(downloaded_img.as_slice())
            .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?
            .to_rgb8()
    };

    Ok(selected_image)
}

// TODO: add func comment & tests below
fn compute_user_theme_averages(config: &Config) -> Result<Vec<[u8; 3]>, WallpaperGeneratorError> {
    let user_themes = config
        .themes()
        .ok_or(WallpaperGeneratorError::ImageGeneration(
            "No user themes defined in config".to_string(),
        ))?;

    if user_themes.themes().is_empty() {
        return Err(WallpaperGeneratorError::ImageGeneration(
            "User themes are defined in config, but its an empty array.".to_string(),
        ));
    }

    let mut theme_averages: Vec<[u8; 3]> = vec![];
    for theme in user_themes.themes() {
        config
            .print_if_verbose(format!("Computing average color of user theme: {}", theme).as_str());
        let curr_theme_avg = theme
            .to_color_theme()
            .average_color(false)
            .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?;
        theme_averages.push(curr_theme_avg);
    }
    config.print_if_verbose(
        format!(
            "Computed average colors for {} user themes",
            theme_averages.len()
        )
        .as_str(),
    );
    Ok(theme_averages)
}

/// Compares the average color of an image to the average colors of each user theme.
/// Returns an integer where the lower the number is the better. The best possible match is 0.
fn compare_image_to_user_theme_averages(
    config: &Config,
    user_theme_averages: &[[u8; 3]],
    image_average: [u8; 3],
) -> u32 {
    let mut best_distance: u32 = u32::MAX;
    for (i, theme_avg) in user_theme_averages.iter().enumerate() {
        let distance = ((theme_avg[0] as i32 - image_average[0] as i32).pow(2)
            + (theme_avg[1] as i32 - image_average[1] as i32).pow(2)
            + (theme_avg[2] as i32 - image_average[2] as i32).pow(2)) as u32;
        config.print_if_verbose(
            format!(
                "Distance from image average {:?} to theme {} average {:?} is {}",
                image_average,
                i + 1,
                theme_avg,
                distance
            )
            .as_str(),
        );
        if distance < best_distance {
            best_distance = distance;
        }
    }
    best_distance
}

fn download_image_to_memory(
    config: &Config,
    url: &str,
) -> Result<Vec<u8>, WallpaperGeneratorError> {
    config.print_if_verbose(format!("Downloading image from {}", url).as_str());
    let image = reqwest::blocking::get(url)
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?
        .bytes()
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?
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
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?
        .json::<SpotlightResponse>()
        .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))?;
    if res.batchrsp.items.is_empty() {
        return Err(WallpaperGeneratorError::ImageGeneration(
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
            .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))?;
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
