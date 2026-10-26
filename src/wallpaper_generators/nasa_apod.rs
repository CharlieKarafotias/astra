use super::super::configuration::Config;
use super::utils::{AstraImage, WallpaperGeneratorError, download_image_to_memory};
use chrono::{Local, NaiveDate};
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Deserialize;
use std::borrow::Cow;

/// Today's picture of the day. This page also links to the calendar and archive.
const APOD_TODAY_URL: &str = "https://science.nasa.gov/apod/";

/// Returns a month of APOD entries as calendar HTML. Each day is a
/// `data-date="YYYY-MM-DD"` cell containing the article link.
const APOD_CALENDAR_URL: &str = "https://science.nasa.gov/wp-json/smd/v1/content-list/";

/// WordPress category id for APOD image articles.
const APOD_CATEGORY_ID: &str = "22766";

/// Category filter required by the content-list endpoint to return APOD entries
/// only (the same value the APOD calendar block sends).
const APOD_CALENDAR_BASE_TERMS: &str =
    r#"{"category":"22766","science-org":"","internal-terms":"","news-tags":""}"#;

/// Renders a random APOD article (the request the random-apod page makes).
const APOD_RANDOM_URL: &str = "https://science.nasa.gov/wp-json/smd/v1/embed-post";

/// Marker for APOD's own image assets on `science.nasa.gov`.
const APOD_ASSET_MARKER: &str = "/cds/apod/apod/";

/// How many random articles to request before giving up. Video entries have no
/// still picture, so a retry usually finds an image.
const APOD_RANDOM_ATTEMPTS: usize = 5;

/// Generates a wallpaper from the NASA Astronomy Picture of the Day website.
///
/// The picture is resolved from the new `science.nasa.gov` site:
///   - Today's picture from <https://science.nasa.gov/apod/>;
///   - Any other date through the APOD calendar API
///     (`wp-json/smd/v1/content-list/`), which maps the date to its image
///     article, whose page links the original asset;
///   - A random picture through the `random-apod` embed endpoint.
///
/// The page's `<a href>` to the full-resolution asset is used first, then the
/// rendered `<img src>` (mapped back to the original `content/dam` file).
/// Video entries have no still picture of the day and return a clear error.
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
    random: bool,
) -> Result<AstraImage, WallpaperGeneratorError> {
    config.print_if_verbose("Generating NASA Astronomy Picture of the Day...");

    let html = if random && !config.respect_user_config {
        config.print_if_verbose("Using a random Astronomy Picture of the Day...");
        download_random_apod_html()?
    } else {
        let img_date = resolve_apod_date(config, date)?;
        download_apod_page(config, &img_date)?
    };

    let img_download_link = retrieve_image_download_url(&html)?;
    let downloaded_img = download_image_to_memory(config, &img_download_link)?;
    Ok(image::load_from_memory(downloaded_img.as_slice())
        .map_err(|e| WallpaperGeneratorError::ImageGeneration(e.to_string()))?
        .to_rgb8())
}

/// Picks the date to use: the configured random range, the date passed on the
/// command line, or today.
fn resolve_apod_date(
    config: &Config,
    date: &Option<ApodDate>,
) -> Result<ApodDate, WallpaperGeneratorError> {
    if config.respect_user_config {
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
            (true, true) => ApodDate::today().map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive today's date as APOD Date: {e}"
                ))
            }),
            (false, true) => ApodDate::random_between(Some(&date_from), None).map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive random date between {date_from} and today: {e}"
                ))
            }),
            (true, false) => ApodDate::random_between(None, Some(&date_to)).map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "Failed to derive random date between earlest NASA apod date and {date_to}: {e}"
                ))
            }),
            (false, false) => {
                ApodDate::random_between(Some(&date_from), Some(&date_to)).map_err(|e| {
                    WallpaperGeneratorError::ImageGeneration(format!(
                        "Failed to derive random date between {date_from} and {date_to}: {e}"
                    ))
                })
            }
        }
    } else if let Some(d) = date {
        config.print_if_verbose(format!("Using received date from args - {d}").as_str());
        Ok(d.clone())
    } else {
        config.print_if_verbose("Using today's date");
        ApodDate::today().map_err(|e| {
            WallpaperGeneratorError::ImageGeneration(format!(
                "Failed to derive today's date as APOD Date: {e}"
            ))
        })
    }
}

/// Fetches the HTML page that contains the requested day's picture.
///
/// Today's picture is on the APOD landing page. Other dates are looked up in
/// the calendar API first, because the new article URLs contain a title slug
/// and cannot be built from the date alone.
fn download_apod_page(config: &Config, date: &ApodDate) -> Result<String, WallpaperGeneratorError> {
    let today = ApodDate::today().map_err(|e| {
        WallpaperGeneratorError::ImageGeneration(format!(
            "Failed to derive today's date as APOD Date: {e}"
        ))
    })?;

    if date == &today {
        config.print_if_verbose(format!("Fetching today's APOD from {APOD_TODAY_URL}").as_str());
        return download_page_html(APOD_TODAY_URL);
    }

    let article_url = find_apod_article_url(config, date)?;
    config.print_if_verbose(format!("Fetching APOD article {article_url}").as_str());
    download_page_html(&article_url)
}

/// Looks up the image article for a date in the APOD calendar API.
fn find_apod_article_url(
    config: &Config,
    date: &ApodDate,
) -> Result<String, WallpaperGeneratorError> {
    let naive = date.to_date().map_err(|e| {
        WallpaperGeneratorError::ImageGeneration(format!("Invalid APOD date {date}: {e}"))
    })?;
    let year = naive.format("%Y").to_string();
    let month = naive.format("%m").to_string();
    let day = naive.format("%Y-%m-%d").to_string();

    let mut url = reqwest::Url::parse(APOD_CALENDAR_URL)
        .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))?;
    url.query_pairs_mut()
        .append_pair("post_types", "image-article")
        .append_pair("layout", "calendar")
        .append_pair("response_format", "html")
        .append_pair("order", "DESC")
        .append_pair("orderby", "date")
        .append_pair("number_of_items", "2000")
        .append_pair("current_page", "1")
        .append_pair("calendar_year", &year)
        .append_pair("calendar_month", &month)
        .append_pair("base_terms", APOD_CALENDAR_BASE_TERMS);

    let response = reqwest::blocking::Client::new()
        .get(url)
        .send()
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))?;
    if !status.is_success() {
        return Err(WallpaperGeneratorError::Network(format!(
            "APOD calendar request failed with status {status}"
        )));
    }

    config
        .print_if_verbose(format!("Looking up {day} in the {year}-{month} APOD calendar").as_str());

    let calendar: ApodCalendarResponse = serde_json::from_str(&body).map_err(|e| {
        WallpaperGeneratorError::Parse(format!("Failed to parse the APOD calendar response: {e}"))
    })?;

    find_calendar_entry(&calendar.content, &day).ok_or_else(|| {
        WallpaperGeneratorError::ImageGeneration(format!(
            "NASA has no Astronomy Picture of the Day for {day} - try a different day or another generator"
        ))
    })
}

/// Finds the image article link for `day` in the calendar HTML returned by the
/// content-list endpoint.
fn find_calendar_entry(calendar_html: &str, day: &str) -> Option<String> {
    let marker = format!("data-date=\"{day}\"");
    let cell_start = calendar_html.find(&marker)? + marker.len();
    let cell_end = calendar_html[cell_start..]
        .find("data-date=\"")
        .map_or(calendar_html.len(), |offset| cell_start + offset);
    let cell = &calendar_html[cell_start..cell_end];

    let mut rest = cell;
    while let Some(index) = rest.find("href=\"") {
        let after = &rest[index + "href=\"".len()..];
        let end = after.find('"')?;
        let url = &after[..end];
        if url.contains("/image-article/") {
            return Some(url.to_string());
        }
        rest = after;
    }
    None
}

/// Downloads a random APOD article, retrying when the chosen entry is a video
/// (which has no still picture to extract).
fn download_random_apod_html() -> Result<String, WallpaperGeneratorError> {
    let mut last_error = None;
    for _ in 0..APOD_RANDOM_ATTEMPTS {
        let html = request_random_apod_html()?;
        match retrieve_image_download_url(&html) {
            Ok(_) => return Ok(html),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        WallpaperGeneratorError::ImageGeneration(
            "Could not find a random Astronomy Picture of the Day with an image - try again or use another generator"
                .to_string(),
        )
    }))
}

/// Requests the same embed the `random-apod` page uses.
fn request_random_apod_html() -> Result<String, WallpaperGeneratorError> {
    let body = serde_json::json!({
        "attributes": {
            "selectedPostTypes": ["image-article"],
            "selectedCategory": APOD_CATEGORY_ID,
            "displayFeaturedImageAndTitleOnly": false,
            "randomizePost": true,
        },
        "currentPostId": 0,
    });

    let response = reqwest::blocking::Client::new()
        .post(APOD_RANDOM_URL)
        .json(&body)
        .send()
        .map_err(|e| WallpaperGeneratorError::Network(e.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|e| WallpaperGeneratorError::Parse(e.to_string()))?;
    if !status.is_success() {
        return Err(WallpaperGeneratorError::Network(format!(
            "Random APOD request failed with status {status}"
        )));
    }

    let random: ApodRandomResponse = serde_json::from_str(&body).map_err(|e| {
        WallpaperGeneratorError::Parse(format!("Failed to parse the random APOD response: {e}"))
    })?;

    if random.html.is_empty() {
        return Err(WallpaperGeneratorError::Parse(
            "The random APOD response was empty".to_string(),
        ));
    }
    Ok(random.html)
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

#[derive(Deserialize)]
struct ApodCalendarResponse {
    content: String,
}

#[derive(Deserialize)]
struct ApodRandomResponse {
    html: String,
}

/// Extracts the URL of the picture of the day.
///
/// 1. `<a href=".../cds/apod/apod/...">` — the link to the full-resolution asset.
/// 2. `<img src=".../cds/apod/apod/...">` — the rendered image, for pages that do
///    not wrap it in a link. `dynamicimage` copies are mapped back to the
///    original `content/dam` file and the resize/crop query is dropped.
///
/// The landing page's `og:image` is a generic APOD banner rather than the
/// picture of the day, so it is deliberately not used as a fallback.
///
/// Video entries (YouTube/Vimeo embeds) have no still picture of the day, so they
/// return an error instead of downloading an unrelated thumbnail.
fn retrieve_image_download_url(webpage_html: &str) -> Result<String, WallpaperGeneratorError> {
    if is_video_page(webpage_html) {
        return Err(WallpaperGeneratorError::ImageGeneration(
            "This Astronomy Picture of the Day is a video - try a different day or another generator"
                .to_string(),
        ));
    }

    find_apod_asset_url(webpage_html, "href")
        .or_else(|| find_apod_asset_url(webpage_html, "src"))
        .ok_or_else(|| {
            WallpaperGeneratorError::ImageGeneration(
                "Today's Astronomy Picture of the Day does not have a downloadable image - try a different day or another generator".to_string()
            )
        })
}

/// Whether the page embeds a video rather than a picture.
fn is_video_page(html: &str) -> bool {
    html.contains("youtube.com/embed")
        || html.contains("player.vimeo.com")
        || html.contains("property=\"og:video\"")
}

/// Finds the first `attribute="..."` URL pointing at an APOD image asset.
///
/// `href` links point at the original, highest-resolution file; `src` is the
/// rendered `<img>` and is the fallback for pages that do not link the image.
/// `/jcr:content/` URLs are resized renditions, so they are skipped.
fn find_apod_asset_url(html: &str, attribute: &str) -> Option<String> {
    let needle = format!("{attribute}=\"");
    let mut rest = html;
    while let Some(index) = rest.find(&needle) {
        let after = &rest[index + needle.len()..];
        let end = after.find('"')?;
        let url = &after[..end];
        if url.contains(APOD_ASSET_MARKER) && !url.contains("/jcr:content/") {
            return Some(original_asset_url(url));
        }
        rest = &after[end..];
    }
    None
}

/// Returns the original asset URL for an APOD image link.
///
/// `dynamicimage` URLs are resized copies, so they are mapped to the
/// `content/dam` original. Query parameters (`?w=...&h=...`) are renditions and
/// are dropped as well.
fn original_asset_url(url: &str) -> String {
    strip_query(url).replace("/dynamicimage/assets/", "/content/dam/")
}

/// Drops `?query` from a URL, keeping the path to the original asset.
fn strip_query(url: &str) -> &str {
    url.split_once('?').map_or(url, |(path, _)| path)
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
            month: 6,
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

impl JsonSchema for ApodDate {
    fn schema_name() -> Cow<'static, str> {
        "ApodDate".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^[0-9]{6}$",
            "description": "A date in YYMMDD format, for example \"260423\" for April 23rd, 2026.",
            "examples": ["260423"]
        })
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

    if !(1..=12).contains(&month) {
        return Err(ApodDateError::InvalidMonth(
            "Month must be between 01 and 12".to_string(),
        ));
    }
    if !(1..=31).contains(&day) {
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
    fn test_retrieve_image_download_url_dynamic_image_maps_to_original() {
        // The landing page renders a resized `dynamicimage` copy without linking it.
        let html = r#"<img width="1960" height="2048" src="https://assets.science.nasa.gov/dynamicimage/assets/science/cds/apod/apod/2026/october/OrionSword_Palaia_2000.jpg?w=1960&amp;h=2048&amp;fit=clip&amp;crop=faces%2Cfocalpoint" />"#;

        let result = retrieve_image_download_url(html).unwrap();
        assert_eq!(
            result,
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2026/october/OrionSword_Palaia_2000.jpg"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_skips_renditions() {
        // The first match is a resized rendition; the original must win.
        let html = r#"
            <a href="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2025/january/AlphaCen_Cantrell_960.jpg/jcr:content/renditions/cq5dam.web.1280.1280.jpeg">rendition</a>
            <a href="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2025/january/AlphaCen_Cantrell_960.jpg">original</a>
        "#;

        let result = retrieve_image_download_url(html).unwrap();
        assert_eq!(
            result,
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2025/january/AlphaCen_Cantrell_960.jpg"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_linked_full_resolution_asset() {
        // Current pages wrap the rendered image in a link to the original file.
        let html = r#"
            <meta property="og:image" content="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2024/january/ngc1232b_vlt_3969.jpg/jcr:content/renditions/cq5dam.web.1280.1280.jpeg" />
            <figure>
                <a href="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2024/january/ngc1232b_vlt_3969.jpg">
                    <img src="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2024/january/ngc1232b_vlt_3969.jpg?w=960&amp;h=909&amp;fit=clip" />
                </a>
            </figure>
        "#;

        let result = retrieve_image_download_url(html).unwrap();
        assert_eq!(
            result,
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2024/january/ngc1232b_vlt_3969.jpg"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_img_src_without_link() {
        // Some pages render the image without wrapping it in an <a>.
        let html = r#"<img width="960" src="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2025/january/AlphaCen_Cantrell_960.jpg?w=960&amp;h=909&amp;fit=clip" />"#;

        let result = retrieve_image_download_url(html).unwrap();
        assert_eq!(
            result,
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2025/january/AlphaCen_Cantrell_960.jpg"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_ignores_og_image_banner() {
        // The landing page's og:image is the APOD banner, not the picture of the day.
        let html = r#"<meta property="og:image" content="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2026/june/eagle.jpg/jcr:content/renditions/cq5dam.web.1280.1280.jpeg" />"#;

        let err = retrieve_image_download_url(html).unwrap_err();

        match err {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                assert!(msg.contains("does not have a downloadable image"))
            }
            e => panic!("Expected ImageGeneration error: {e}"),
        }
    }

    #[test]
    fn test_retrieve_image_download_url_old_gif() {
        // 1995-2002 entries are still GIFs, so the gif decoder must stay enabled.
        let html = r#"
            <a href="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/1995/june/pleiades2.gif" target="_blank">
                <img src="https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/1995/june/pleiades2.gif" />
            </a>
        "#;

        let result = retrieve_image_download_url(html).unwrap();
        assert_eq!(
            result,
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/1995/june/pleiades2.gif"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_video_page() {
        let html = r#"
            <meta property="og:video" content="https://www.youtube.com/v/dY6poe072_c" />
            <iframe src="https://www.youtube.com/embed/dY6poe072_c"></iframe>
        "#;

        let err = retrieve_image_download_url(html).unwrap_err();

        match err {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                assert!(msg.contains("is a video"), "unexpected message: {msg}")
            }
            e => panic!("Expected ImageGeneration error: {e}"),
        }
    }

    #[test]
    fn test_find_calendar_entry_returns_the_requested_day() {
        let calendar = r#"
            <div class="calendar-cell has-posts" data-date="2025-01-01" aria-label="January 1, 2025: 1 post">
                <a href="https://science.nasa.gov/image-article/apod-2025-january-1-alpha-centauri/" class="calendar-post-tile">1</a>
            </div>
            <div class="calendar-cell has-posts" data-date="2025-01-02" aria-label="January 2, 2025: 1 post">
                <a href="https://science.nasa.gov/image-article/apod-2025-january-2-the-sun/" class="calendar-post-tile">2</a>
            </div>
        "#;

        assert_eq!(
            find_calendar_entry(calendar, "2025-01-01").as_deref(),
            Some("https://science.nasa.gov/image-article/apod-2025-january-1-alpha-centauri/")
        );
        assert_eq!(
            find_calendar_entry(calendar, "2025-01-02").as_deref(),
            Some("https://science.nasa.gov/image-article/apod-2025-january-2-the-sun/")
        );
    }

    #[test]
    fn test_find_calendar_entry_without_a_post() {
        // APOD starts on 1995 June 20; earlier days are empty calendar cells.
        let calendar = r#"
            <div class="calendar-cell" data-date="1995-06-01" aria-label="June 1, 1995: No posts"></div>
            <div class="calendar-cell has-posts" data-date="1995-06-20" aria-label="June 20, 1995: 1 post">
                <a href="https://science.nasa.gov/image-article/apod-1995-june-20-pleiades-star-cluster/">20</a>
            </div>
        "#;

        assert_eq!(find_calendar_entry(calendar, "1995-06-01"), None);
        assert_eq!(
            find_calendar_entry(calendar, "1995-06-20").as_deref(),
            Some("https://science.nasa.gov/image-article/apod-1995-june-20-pleiades-star-cluster/")
        );
    }

    #[test]
    fn test_find_calendar_entry_unknown_day() {
        let calendar = r#"<div class="calendar-cell" data-date="2025-01-01"></div>"#;

        assert_eq!(find_calendar_entry(calendar, "2025-02-01"), None);
    }

    #[test]
    fn test_original_asset_url_maps_dynamic_image_host() {
        assert_eq!(
            original_asset_url(
                "https://assets.science.nasa.gov/dynamicimage/assets/science/cds/apod/apod/2026/october/OrionSword_Palaia_2000.jpg?w=1960&amp;h=2048&amp;fit=clip"
            ),
            "https://assets.science.nasa.gov/content/dam/science/cds/apod/apod/2026/october/OrionSword_Palaia_2000.jpg"
        );
    }

    #[test]
    fn test_retrieve_image_download_url_missing_tag() {
        let html = "<html><body>No image here</body></html>";

        let err = retrieve_image_download_url(html).unwrap_err();

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

        let err = retrieve_image_download_url(html).unwrap_err();

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
