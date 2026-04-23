use super::super::configuration::Config;
use super::utils::{AstraImage, WallpaperGeneratorError, download_image_to_memory};

/// Generates a wallpaper from the NASA Astronomy Picutre of the Day website. The website
/// provides a photo of the day, which is used as the wallpaper.
/// The image download URL is derived by:
///   - Going to https://apod.nasa.gov/apod/ap{date in format yymmdd}.html (April 23, 2026 -> 260423)
///   - HTML inspected for an <img> tag with `src`
///   - Navigate to `src` which is format like `https://apod.nasa.gov/apod/image/2604/noirlab2610c_1024.jpg`
///   - Downloaded image from the URL
///
/// The image is saved to astra wallpaper folder with name of the form
/// `nasa_apod_<unix_timestamp>.png` (if save and update are true).
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
pub fn generate_nasa_apod(config: &Config) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating NASA Astronomy Picture of the Day...");

    if config.respect_user_config {
        // TODO: setup nasa_apod_gen config
        config.print_if_verbose("User config detected with nasa_apod_gen options...");
    }

    // TODO: derive current date here (or optional other date via param args)
    let base = "https://apod.nasa.gov/apod";
    let date = "260423";
    let url = format!("{base}/ap{}.html", date);

    let html = download_page_html(&url)?;
    let img_download_link = format!("{base}/{}", retrieve_image_download_url(html)?);
    let downloaded_img = download_image_to_memory(config, &img_download_link)?;
    Ok(image::load_from_memory(downloaded_img.as_slice())
        .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?
        .to_rgb8())
}

/// Helper to download the html content of a webpage
///
/// # Errors
///
/// This function returns a `Result` containing the HTML of the webpage provided.
/// There are 2 possible erorrs:
///
/// * `Network`: The GET call to the URL fails
/// * `Parse`: The text representation of the response body fails
fn download_page_html(url: &str) -> Result<String, WallpaperGeneratorError> {
    reqwest::blocking::get(url)
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?
        .text()
        .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))
}

fn retrieve_image_download_url(webpage_html: String) -> Result<String, WallpaperGeneratorError> {
    let pos_of_image_src = webpage_html.find("<a href=\"image/").ok_or(WallpaperGeneratorError::ImageGeneration("Today's Astronomy Picture of the Day does not have a downloadable image - try a different day or another generator".to_string()))?;
    let (_, link) = webpage_html.split_at(pos_of_image_src);

    // Take second place which has the link
    let mut separated = link.split("\"");
    separated.next();
    Ok(separated
        .next()
        .ok_or(WallpaperGeneratorError::Parse(
            "Expected a href URL here, received None. Report this error to developer".to_string(),
        ))?
        .to_string())
}
