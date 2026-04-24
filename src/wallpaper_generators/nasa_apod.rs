use super::super::configuration::Config;
use super::utils::{AstraImage, WallpaperGeneratorError, download_image_to_memory};
use chrono::{Local, NaiveDate};
use serde::Deserialize;

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
/// * `ImageGeneration`: The image failed to download, the image
///   failed to parse or APOD date failure.
/// * `Network`: The API request failed.
pub fn generate_nasa_apod(
    config: &Config,
    date: &Option<ApodDate>,
) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating NASA Astronomy Picture of the Day...");

    let img_date: &ApodDate = if config.respect_user_config {
        config.print_if_verbose("User config detected with nasa_apod_gen options...");
        let date_from = crate::respect_user_config_or_default!(
            config,
            nasa_apod_gen,
            date_from,
            Ok(String::new())
        )?;
        let date_to = crate::respect_user_config_or_default!(
            config,
            nasa_apod_gen,
            date_to,
            Ok(String::new())
        )?;

        match (date_from.is_empty(), date_to.is_empty()) {
            (true, true) => &ApodDate::today().map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive today's date as APOD Date: {e}"
                ))
            })?,
            (false, true) => &ApodDate::random_between(Some(&date_from), None).map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive random date between {date_from} and today: {e}"
                ))
            })?,
            (true, false) => &ApodDate::random_between(None, Some(&date_to)).map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive random date between earlest NASA apod date and {date_to}: {e}"
                ))
            })?,
            (false, false) => {
                &ApodDate::random_between(Some(&date_from), Some(&date_to)).map_err(|e| {
                    WallpaperGeneratorError::ImageGeneration(format!(
                        "Failed to derive random date between {date_from} and {date_to}: {e}"
                    ))
                })?
            }
        }
    } else if let Some(d) = date {
        config.print_if_verbose(format!("Using received date from args - {d}").as_str());
        d
    } else {
        config.print_if_verbose("Using today's date");
        &ApodDate::today().map_err(|e| {
            WallpaperGeneratorError::ImageGeneration(format!(
                "Failed to derive today's date as APOD Date: {e}"
            ))
        })?
    };

    let base = "https://apod.nasa.gov/apod";
    let url = format!("{base}/ap{}.html", img_date);

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
    let pos_of_image_src = webpage_html
        .find("href=\"image/")
        .ok_or(WallpaperGeneratorError::ImageGeneration(
            "Today's Astronomy Picture of the Day does not have a downloadable image - try a different day or another generator".to_string()
        ))?;
    let (_, link) = webpage_html.split_at(pos_of_image_src);

    // Take second place which has the link
    let mut separated = link.split('"');
    separated.next();
    Ok(separated
        .next()
        .ok_or(WallpaperGeneratorError::Parse(
            "Expected a href URL here, received None. Report this error to developer".to_string(),
        ))?
        .to_string())
}

#[derive(Clone, Debug, PartialEq)]
pub struct ApodDate {
    year: u32,
    month: u32,
    day: u32,
}

impl ApodDate {
    // Creates the earliest APOD date possible (June 20, 1995)
    fn earliest() -> Self {
        ApodDate {
            year: 95,
            month: 06,
            day: 20,
        }
    }

    // Sets the ApodDate to today
    fn today() -> Result<Self, ApodDateError> {
        let now = Local::now();
        let formatted = now.format("%y%m%d").to_string();
        Ok(Self {
            year: formatted[0..2]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidYear(e.to_string()))?,
            month: formatted[2..4]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidMonth(e.to_string()))?,
            day: formatted[4..6]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidDay(e.to_string()))?,
        })
    }

    fn random_between(from: Option<&str>, to: Option<&str>) -> Result<Self, ApodDateError> {
        let df = from.map(parse_yymmdd).unwrap_or(Ok(Self::earliest()))?;
        let dt = to.map(parse_yymmdd).unwrap_or(Ok(Self::today()?))?;
        let df_date_epoch = df.to_date()?.to_epoch_days();
        let dt_date_epoch = dt.to_date()?.to_epoch_days();
        let rand_epoch = rand::random_range(df_date_epoch..=dt_date_epoch);
        let rand_as_date = NaiveDate::from_epoch_days(rand_epoch).ok_or(ApodDateError::Chrono(
            format!("Failed to cast rand_epoch '{rand_epoch}' to NaiveDate"),
        ))?;
        let formatted = rand_as_date.format("%y%m%d").to_string();
        Ok(Self {
            year: formatted[0..2]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidYear(e.to_string()))?,
            month: formatted[2..4]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidMonth(e.to_string()))?,
            day: formatted[4..6]
                .parse::<u32>()
                .map_err(|e| ApodDateError::InvalidDay(e.to_string()))?,
        })
    }

    fn to_date(&self) -> Result<NaiveDate, ApodDateError> {
        let year = if self.year >= 95 && self.year <= 99 {
            1900 + self.year
        } else {
            2000 + self.year
        };
        Ok(
            NaiveDate::from_ymd_opt(year as i32, self.month, self.day).ok_or(
                ApodDateError::Chrono(
                    "Cast to NaiveDate failed, ensure month/day are valid".into(),
                ),
            ),
        )?
    }
}

impl<'de> Deserialize<'de> for ApodDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match parse_yymmdd(&s) {
            Ok(apod) => Ok(apod),
            Err(e) => Err(serde::de::Error::custom(e)),
        }
    }
}

impl std::fmt::Display for ApodDate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}{:02}{:02}", self.year, self.month, self.day)
    }
}

pub fn parse_yymmdd(s: &str) -> Result<ApodDate, ApodDateError> {
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_digit()) {
        return Err(ApodDateError::Parse(
            "Date must be exactly 6 digits in YYMMDD format".to_string(),
        ));
    }

    let year = s[0..2]
        .parse::<u32>()
        .map_err(|e| ApodDateError::InvalidYear(e.to_string()))?;
    let month = s[2..4]
        .parse::<u32>()
        .map_err(|e| ApodDateError::InvalidMonth(e.to_string()))?;
    let day = s[4..6]
        .parse::<u32>()
        .map_err(|e| ApodDateError::InvalidDay(e.to_string()))?;

    if month < 1 || month > 12 {
        return Err(ApodDateError::InvalidMonth(
            "Month must be between 01 and 12".to_string(),
        ));
    }
    if day < 1 || day > 31 {
        return Err(ApodDateError::InvalidDay(
            "Day must be between 01 and 31".to_string(),
        ));
    }

    Ok(ApodDate { year, month, day })
}

#[derive(Debug, PartialEq)]
pub enum ApodDateError {
    Chrono(String),
    InvalidYear(String),
    InvalidMonth(String),
    InvalidDay(String),
    Parse(String),
}

impl std::fmt::Display for ApodDateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApodDateError::Chrono(msg) => write!(f, "Chrono Error: {}", msg),
            ApodDateError::InvalidMonth(msg) => write!(f, "Invalid Month: {}", msg),
            ApodDateError::InvalidYear(msg) => write!(f, "Invalid Year: {}", msg),
            ApodDateError::InvalidDay(msg) => write!(f, "Invalid Day: {}", msg),
            ApodDateError::Parse(msg) => write!(f, "APOD Parse Error: {}", msg),
        }
    }
}

impl std::error::Error for ApodDateError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_yymmdd_valid() {
        let result = parse_yymmdd("260423").unwrap();
        assert_eq!(
            result,
            ApodDate {
                year: 26,
                month: 4,
                day: 23
            }
        );
    }

    #[test]
    fn test_parse_yymmdd_invalid_length() {
        let err = parse_yymmdd("12345").unwrap_err();
        assert_eq!(
            err,
            ApodDateError::Parse("Date must be exactly 6 digits in YYMMDD format".to_string())
        );
    }

    #[test]
    fn test_parse_yymmdd_non_numeric() {
        let err = parse_yymmdd("26AB23").unwrap_err();
        assert_eq!(
            err,
            ApodDateError::Parse("Date must be exactly 6 digits in YYMMDD format".to_string())
        );
    }

    #[test]
    fn test_parse_yymmdd_invalid_month() {
        let err = parse_yymmdd("261323").unwrap_err();
        assert_eq!(
            err,
            ApodDateError::InvalidMonth("Month must be between 01 and 12".to_string())
        );
    }

    #[test]
    fn test_parse_yymmdd_invalid_day() {
        let err = parse_yymmdd("260432").unwrap_err();
        assert_eq!(
            err,
            ApodDateError::InvalidDay("Day must be between 01 and 31".to_string())
        );
    }

    #[test]
    fn test_apod_date_display() {
        let date = ApodDate {
            year: 6,
            month: 4,
            day: 3,
        };

        assert_eq!(date.to_string(), "060403");
    }

    #[test]
    fn test_retrieve_image_download_url_valid() {
        let html = r#"
            <html>
                <body>
                    <a href="image/2604/test_image.jpg">Link</a>
                </body>
            </html>
        "#;

        let result = retrieve_image_download_url(html.to_string()).unwrap();
        assert_eq!(result, "image/2604/test_image.jpg");
    }

    #[test]
    fn test_retrieve_image_download_url_missing_tag() {
        let html = "<html><body>No image here</body></html>";

        let err = retrieve_image_download_url(html.to_string()).unwrap_err();

        match err {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                assert!(msg.contains("does not have a downloadable image"))
            }
            _ => panic!("Expected ImageGeneration error"),
        }
    }

    #[test]
    fn test_retrieve_image_download_url_malformed_href() {
        let html = r#"
            <html>
                <body>
                    <a href=>Broken</a>
                </body>
            </html>
        "#;

        let err = retrieve_image_download_url(html.to_string()).unwrap_err();

        match err {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                assert!(
                msg.contains(
                    "Today's Astronomy Picture of the Day does not have a downloadable image - try a different day or another generator"
                ))
            }
            e => panic!("Expected Parse error: {e}"),
        }
    }

    #[test]
    fn test_apod_date_today_format() {
        let today = ApodDate::today().unwrap();
        let formatted = today.to_string();

        assert_eq!(formatted.len(), 6);
        assert!(formatted.chars().all(|c| c.is_ascii_digit()));
    }
}
