use super::super::{
    configuration::{Config, Frequency},
    constants::{APPLICATION, ORGANIZATION, QUALIFIER},
};
use crate::cli::Generator;
use crate::os_implementations::update_wallpaper;
use directories::ProjectDirs;
use image::{ImageBuffer, Rgb};
use plist::Value;
use std::{
    error::Error,
    fmt,
    fs::{self, create_dir_all, read_dir, remove_dir_all, remove_file, rename},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub type AstraImage = ImageBuffer<Rgb<u8>, Vec<u8>>;

/// Creates a folder named "wallpapers" under the data_dir folder of Astra.
/// For each path, see: https://lib.rs/crates/directories
///
/// # Returns
///
/// A `Result` containing the path to the created folder on success, or a
/// `WallpaperGeneratorError` on failure.
pub(super) fn create_wallpaper_folder() -> Result<PathBuf, WallpaperGeneratorError> {
    let proj_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .ok_or_else(|| WallpaperGeneratorError::OS("could not derive data_dir".to_string()))?;
    let path = proj_dirs.data_dir().join("Wallpapers");
    create_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
    Ok(path)
}

/// Deletes wallpapers from the "wallpapers" folder.
/// For each path, see: https://lib.rs/crates/directories
///
/// # Arguments
///
/// * `delete_all` - If true, deletes all wallpapers and the "astra_wallpapers" folder.
/// * `delete_dir` - If true, deletes the "astra_wallpapers" folder.
/// * `older_than` - If set, deletes wallpapers older than the specified frequency.
///
/// # Returns
///
/// A `Result` containing `()` on success, or a `WallpaperGeneratorError` on failure.
pub fn delete_wallpapers(
    config: &Config,
    delete_all: bool,
    delete_dir: bool,
    older_than: Option<&Frequency>,
) -> Result<(), WallpaperGeneratorError> {
    let path = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
        .map(|dirs| dirs.data_dir().join("Wallpapers"))
        .ok_or_else(|| {
            WallpaperGeneratorError::OS("could not derive wallpapers path".to_string())
        })?;

    // NOTE: if user has never run astra before, the directory might not exist
    if !path.is_dir() {
        config.print_if_verbose(
            "WARN: wallpaper directory does not exist. Skipping wallpaper cleanup",
        );
        // The directory being gone means every Astra gallery entry is stale.
        #[cfg(target_os = "macos")]
        crate::os_implementations::prune_stale_wallpaper_gallery_entries(config, &path);
        return Ok(());
    }

    config.print_if_verbose(format!("Deleting wallpapers from {}", path.display()).as_str());
    if delete_dir {
        remove_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        config.print_if_verbose(
            format!(
                "Deleted all images and directory {} successfully",
                path.display()
            )
            .as_str(),
        );
    } else if delete_all {
        remove_dir_all(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        create_wallpaper_folder().map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
        config.print_if_verbose(
            format!(
                "Deleted all images from directory {} successfully",
                path.display()
            )
            .as_str(),
        );
    } else if let Some(frequency) = older_than {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?
            .as_secs();
        let older_than_sec = frequency.to_seconds();
        config.print_if_verbose(format!("Deleting images older than {}", &frequency).as_str());
        let oldest_timestamp_to_keep = now - older_than_sec;
        for entry in read_dir(&path).map_err(|e| WallpaperGeneratorError::OS(e.to_string()))? {
            let entry = entry.map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            let Some(timestamp) = wallpaper_timestamp(&file_name) else {
                config.print_if_verbose(format!(
                    "ERROR: Encountered file that is not an astra formatted image, skipping file... {}",
                    entry.path().display()
                ).as_str());
                continue;
            };
            if timestamp < oldest_timestamp_to_keep {
                remove_file(entry.path())
                    .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;
                config.print_if_verbose(
                    format!("Deleted image {} successfully", entry.path().display()).as_str(),
                );
            }
        }
    }

    // On macOS, also prune stale Astra entries from the System Settings wallpaper
    // gallery. This never fails the clean; see prune_stale_wallpaper_gallery_entries.
    #[cfg(target_os = "macos")]
    crate::os_implementations::prune_stale_wallpaper_gallery_entries(config, &path);

    Ok(())
}

/// Extracts the Unix timestamp from an Astra wallpaper file name
/// (`<prefix>_<unix_timestamp>.png`).
///
/// Returns `None` for anything that does not match that shape, including names
/// shorter than `_<timestamp>.png`, names without an underscore, non-numeric
/// timestamps, and names that are not valid UTF-8 (the caller stringifies with
/// `to_string_lossy`). `delete_wallpapers` skips those files, so a stray file in the
/// wallpapers directory can never panic or abort a clean; previously the timestamp
/// was sliced by byte index, which panicked on short names and could split a UTF-8
/// character.
///
/// Legacy `astra_1.png`/`astra_2.png` files (from the old macOS slot scheme) parse as
/// timestamps 1 and 2, so they are deleted by the next `--older-than` clean.
fn wallpaper_timestamp(file_name: &str) -> Option<u64> {
    let stem = file_name.strip_suffix(".png")?;
    let (_, timestamp) = stem.rsplit_once('_')?;
    timestamp.parse().ok()
}

/// Removes stale Astra wallpapers from the macOS System Settings "Your Photos"
/// gallery.
///
/// macOS records every image set as a wallpaper in WallpaperAgent's view-model
/// cache, which is what System Settings → Wallpaper → "Your Photos" renders. Items
/// are never dropped when the underlying file disappears, so the gallery grows by
/// one entry per render. This removes only items whose `id.id` points at a file
/// under `wallpapers_dir` that no longer exists. The currently-set wallpaper,
/// system pictures, other apps' images, and the synthetic "Add Photo" / shuffle
/// entries are always kept.
///
/// # Where to look when debugging
///
/// The cache is a binary plist; `plutil -p <path>` prints it readably.
/// - wallpaper path: `viewModel.groups[].items[].id.id` (usually a plain path, but
///   `file://` URLs and percent-encoded names are normalized too);
/// - thumbnail path: `thumbnail.image.url.relative` (a `file://` URL pointing into
///   the image extension's `Caches` directory).
///
/// Thumbnails are only deleted for items that were removed, and only when the path
/// is under `thumbnails_dir`, so a malformed cache can never delete files
/// elsewhere. They are deleted after the cache is rewritten: if the rewrite fails,
/// leaving a few orphaned thumbnails behind is harmless, whereas deleting them
/// first would leave the cache pointing at missing images.
///
/// The cache is only rewritten when at least one item was removed, and the rewrite
/// is atomic (temporary file + rename), so a failure cannot corrupt it.
///
/// # Failure behavior
///
/// Returns the number of items removed and never fails the caller: a missing file,
/// parse error, or I/O error is reported via verbose logging and yields 0, because
/// this format is undocumented and must never break a clean. Panics are not
/// possible here; every fallible step is handled with `?`-style matching or
/// `Option`.
///
/// When debugging a clean that removed nothing, run with `-v`:
/// - `WARN: could not read macOS wallpaper gallery` → the file could not be read or
///   parsed (check the path and `plutil -p`);
/// - no line at all → nothing matched: confirm the item's `id.id` really is under
///   Astra's `Wallpapers` directory and the file really is gone.
// Only called from the macOS implementation; on other targets this entry point
// (and the helpers below, reachable only from it) is intentionally unused.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn prune_wallpaper_gallery_file(
    config: &Config,
    gallery_path: &Path,
    wallpapers_dir: &Path,
    thumbnails_dir: Option<&Path>,
) -> usize {
    let mut gallery = match Value::from_file(gallery_path) {
        Ok(gallery) => gallery,
        Err(e) => {
            config.print_if_verbose(
                format!(
                    "WARN: could not read macOS wallpaper gallery, skipping gallery cleanup: {e}"
                )
                .as_str(),
            );
            return 0;
        }
    };

    let mut stale_thumbnails = Vec::new();
    let removed = prune_stale_gallery_items(
        &mut gallery,
        wallpapers_dir,
        thumbnails_dir,
        &mut stale_thumbnails,
    );
    if removed == 0 {
        return 0;
    }

    if !write_plist_atomic(config, gallery_path, &gallery, "macOS wallpaper gallery") {
        return 0;
    }
    remove_thumbnails(&stale_thumbnails);

    config.print_if_verbose(
        format!("Removed {removed} stale Astra entries from the macOS wallpaper gallery").as_str(),
    );
    removed
}

/// Removes stale Astra entries from the image wallpaper extension's history.
///
/// The image extension persists every image it has been asked to show in
/// `ChoiceRequests.ImageFiles` inside its preferences plist. Each array element is
/// itself a binary plist (stored as a plist `Data` blob) holding:
/// - `originalURL.relative` — the wallpaper file, as a `file://` URL;
/// - `copyURL.relative` — the thumbnail copy in the extension's `Caches` directory;
/// - `dateAdded` — when the entry was added.
///
/// WallpaperAgent rebuilds the System Settings gallery from this history, so
/// pruning only the view-model cache is undone the next time a wallpaper is set.
/// This drops entries whose `originalURL` is a file under `wallpapers_dir` that no
/// longer exists, plus their thumbnail copies.
///
/// # Why kept entries are not re-serialized
///
/// Only the array is rebuilt; each kept element is cloned as the original `Data`
/// bytes. Re-encoding them would re-write out bookmark blobs and timestamp
/// encodings that these undocumented plists use, so keeping the bytes untouched is
/// both safer and necessary for System Settings to keep recognizing the entry.
///
/// # Freeze the extension first
///
/// `WallpaperImageExtension` holds the history in memory and rewrites this file
/// when it exits. Called while it runs, this prune is silently undone later, so the
/// macOS wrapper freezes both wallpaper services around this call (see
/// `prune_stale_wallpaper_gallery_entries` in the macOS module).
///
/// # Failure behavior
///
/// Returns the number of entries removed. The file is only rewritten when
/// something was removed, and the rewrite is atomic. This never fails the caller
/// and cannot panic: a missing file, a plist parse error, or an I/O error is
/// reported via verbose logging and yields 0.
///
/// # Where to look when debugging
///
/// `plutil -p <prefs>` shows the `ImageFiles` array but the nested blobs only as
/// `<data>`; to read one entry:
///
/// ```text
/// plutil -extract 'ChoiceRequests.ImageFiles.0' raw -o - \
///   ~/Library/Containers/com.apple.wallpaper.extension.image/Data/Library/Preferences/com.apple.wallpaper.extension.image.plist \
///   | plutil -p -
/// ```
///
/// If clean removes nothing, check with `-v` for the matching
/// `WARN: could not read macOS wallpaper gallery history` line, then confirm the
/// entry's `originalURL.relative` is under Astra's `Wallpapers` directory and the
/// file is gone.
// Only called from the macOS implementation; on other targets this entry point
// (and the helpers below, reachable only from it) is intentionally unused.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn prune_wallpaper_extension_image_files(
    config: &Config,
    prefs_path: &Path,
    wallpapers_dir: &Path,
    thumbnails_dir: Option<&Path>,
) -> usize {
    let mut prefs = match Value::from_file(prefs_path) {
        Ok(prefs) => prefs,
        Err(e) => {
            config.print_if_verbose(
                format!(
                    "WARN: could not read macOS wallpaper gallery history, skipping gallery cleanup: {e}"
                )
                .as_str(),
            );
            return 0;
        }
    };
    let Some(image_files) = prefs
        .as_dictionary_mut()
        .and_then(|prefs| prefs.get_mut("ChoiceRequests.ImageFiles"))
        .and_then(Value::as_array_mut)
    else {
        return 0;
    };

    let mut kept = Vec::with_capacity(image_files.len());
    let mut stale_thumbnails = Vec::new();
    let mut removed = 0;
    for entry in image_files.iter() {
        let Some(entry_plist) = stale_history_entry(entry, wallpapers_dir) else {
            kept.push(entry.clone());
            continue;
        };
        if let Some(copy_url) = image_file_url(&entry_plist, "copyURL") {
            let copy_path = normalize_file_url_path(&copy_url);
            if let Some(thumbnails_dir) = thumbnails_dir
                && copy_path.starts_with(thumbnails_dir)
            {
                stale_thumbnails.push(copy_path);
            }
        }
        removed += 1;
    }
    if removed == 0 {
        return 0;
    }

    *image_files = kept;
    if !write_plist_atomic(
        config,
        prefs_path,
        &prefs,
        "macOS wallpaper gallery history",
    ) {
        return 0;
    }
    remove_thumbnails(&stale_thumbnails);

    config.print_if_verbose(
        format!("Removed {removed} stale Astra entries from the macOS wallpaper gallery history")
            .as_str(),
    );
    removed
}

/// Reads a nested `{ "relative": "file://..." }` URL from a gallery history entry.
///
/// Both `originalURL` and `copyURL` in the extension's history use this shape:
///
/// ```text
/// { "originalURL": { "relative": "file:///Users/me/.../solid_1700000000.png" } }
/// ```
///
/// Returns `None` when the key is absent or has an unexpected type; callers treat
/// that as "not ours", never as an error.
// See the note on prune_wallpaper_extension_image_files: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn image_file_url(entry: &Value, key: &str) -> Option<String> {
    Some(
        entry
            .as_dictionary()?
            .get(key)?
            .as_dictionary()?
            .get("relative")?
            .as_string()?
            .to_string(),
    )
}

/// Parses a `ChoiceRequests.ImageFiles` entry, returning the parsed entry only if it
/// points at a file under `wallpapers_dir` which no longer exists.
///
/// The outer value is a `Data` blob containing a binary plist; a non-`Data` element
/// or one that does not parse is treated as "keep" (it is not something Astra wrote,
/// or the format changed and we must not delete blindly). Returning the parsed entry
/// rather than a bool lets the caller read `copyURL` without parsing the blob twice.
// See the note on prune_wallpaper_extension_image_files: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn stale_history_entry(entry: &Value, wallpapers_dir: &Path) -> Option<Value> {
    let Value::Data(entry_bytes) = entry else {
        return None;
    };
    let entry_plist = Value::from_reader(std::io::Cursor::new(entry_bytes.as_slice())).ok()?;
    let original_url = image_file_url(&entry_plist, "originalURL")?;
    is_stale_astra_path(&original_url, wallpapers_dir).then_some(entry_plist)
}

/// Whether the macOS wallpaper gallery or its history references an Astra wallpaper
/// that no longer exists.
///
/// This is a read-only pre-check used by the macOS implementation to avoid freezing
/// and restarting the wallpaper services when a clean has nothing to remove. It
/// reuses the same predicates as the pruners (`prune_stale_gallery_items` and
/// `stale_history_entry`), so it cannot disagree with them about what counts as
/// stale.
///
/// A missing or unparsable file yields `false` (skip pruning, change nothing):
/// reporting "stale" for a file we cannot read would only cause a pointless service
/// restart. Pass `None` for a store that does not exist.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn wallpaper_gallery_has_stale_entries(
    gallery_path: Option<&Path>,
    history_path: Option<&Path>,
    wallpapers_dir: &Path,
) -> bool {
    let gallery_is_stale = gallery_path
        .and_then(|path| Value::from_file(path).ok())
        .is_some_and(|mut gallery| {
            prune_stale_gallery_items(&mut gallery, wallpapers_dir, None, &mut Vec::new()) > 0
        });
    let history_is_stale = history_path
        .and_then(|path| Value::from_file(path).ok())
        .is_some_and(|prefs| {
            prefs
                .as_dictionary()
                .and_then(|prefs| prefs.get("ChoiceRequests.ImageFiles"))
                .and_then(Value::as_array)
                .is_some_and(|entries| {
                    entries
                        .iter()
                        .any(|entry| stale_history_entry(entry, wallpapers_dir).is_some())
                })
        });
    gallery_is_stale || history_is_stale
}

/// Serializes `value` as a binary plist and atomically replaces `path` with it.
///
/// Steps: serialize to memory → write a sibling `<name>.astra-tmp` → copy the
/// original file's permissions onto it (the temp file would otherwise get the
/// process umask) → `rename` over the original. `rename` within a directory is
/// atomic, so a reader never sees a half-written plist. On any failure the original
/// file is left untouched and the temp file is removed; the error is only reported
/// under `-v` and the caller keeps its "never fail a clean" contract.
///
/// Returns whether the file was replaced. Cannot panic: the temp file name falls
/// back to an empty string if `path` has no file name, which then fails the write
/// and is reported rather than unwrapped.
// See the note on prune_wallpaper_extension_image_files: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn write_plist_atomic(config: &Config, path: &Path, value: &Value, description: &str) -> bool {
    let mut serialized = Vec::new();
    if let Err(e) = value.to_writer_binary(&mut serialized) {
        config.print_if_verbose(
            format!("WARN: could not serialize pruned {description}, leaving it unchanged: {e}")
                .as_str(),
        );
        return false;
    }

    let temp_path = path.with_file_name(format!(
        "{}.astra-tmp",
        path.file_name().unwrap_or_default().to_string_lossy()
    ));
    if let Err(e) = fs::write(&temp_path, &serialized) {
        config.print_if_verbose(
            format!("WARN: could not update {description}, leaving it unchanged: {e}").as_str(),
        );
        return false;
    }
    // Match the original permissions; the temp file is created with the default
    // umask, which would otherwise change the file's access mode.
    if let Ok(metadata) = fs::metadata(path) {
        let _ = fs::set_permissions(&temp_path, metadata.permissions());
    }
    if let Err(e) = rename(&temp_path, path) {
        let _ = remove_file(&temp_path);
        config.print_if_verbose(
            format!("WARN: could not update {description}, leaving it unchanged: {e}").as_str(),
        );
        return false;
    }
    true
}

/// Deletes cached thumbnails referenced by removed gallery/history entries.
///
/// Best effort by design: these are disposable cache files, and a failure to delete
/// one must not turn a successful clean into an error. Callers only pass paths that
/// were verified to live inside the extension's `Caches` directory.
// See the note on prune_wallpaper_extension_image_files: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn remove_thumbnails(thumbnails: &[PathBuf]) {
    for thumbnail in thumbnails {
        let _ = remove_file(thumbnail);
    }
}

/// Drops gallery items that point at a deleted Astra wallpaper.
///
/// Walks `viewModel.groups[].items[]` and removes an item only when
/// `gallery_item_path` yields a path that is stale according to
/// `is_stale_astra_path`. Items without a usable path (synthetic "Add Photo" /
/// shuffle entries), items for files that still exist, and items for other apps are
/// kept. Every group is visited because the gallery splits entries across groups
/// (for example "Your Photos" vs "Pictures").
///
/// Returns the number of items removed and appends the thumbnails that belonged to
/// them to `stale_thumbnails`; the caller deletes those after the rewrite succeeds.
/// Pass `None` for `thumbnails_dir` to collect nothing (used by the read-only
/// `wallpaper_gallery_has_stale_entries` check).
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn prune_stale_gallery_items(
    gallery: &mut Value,
    wallpapers_dir: &Path,
    thumbnails_dir: Option<&Path>,
    stale_thumbnails: &mut Vec<PathBuf>,
) -> usize {
    let Some(groups) = gallery
        .as_dictionary_mut()
        .and_then(|view_model| view_model.get_mut("viewModel"))
        .and_then(Value::as_dictionary_mut)
        .and_then(|view_model| view_model.get_mut("groups"))
        .and_then(Value::as_array_mut)
    else {
        return 0;
    };

    let mut removed = 0;
    for group in groups {
        let Some(items) = group
            .as_dictionary_mut()
            .and_then(|group| group.get_mut("items"))
            .and_then(Value::as_array_mut)
        else {
            continue;
        };
        items.retain(|item| {
            let Some(path_str) = gallery_item_path(item) else {
                return true;
            };
            if !is_stale_astra_path(path_str, wallpapers_dir) {
                return true;
            }
            if let Some(thumbnail) = gallery_item_thumbnail_path(item)
                && let Some(thumbnails_dir) = thumbnails_dir
                && thumbnail.starts_with(thumbnails_dir)
            {
                stale_thumbnails.push(thumbnail);
            }
            removed += 1;
            false
        });
    }
    removed
}

/// The absolute wallpaper path stored in a gallery item (`id.id`).
///
/// Item shape (only the fields that matter here):
///
/// ```text
/// {
///   "id": { "id": "/Users/me/Library/.../Wallpapers/solid_1700000000.png" },
///   "localizedName": "solid_1700000000",
///   ...
/// }
/// ```
///
/// Returns `None` for items that do not carry a path (synthetic entries such as
/// "Add Photo" or the shuffle item), which the pruner always keeps.
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn gallery_item_path(item: &Value) -> Option<&str> {
    item.as_dictionary()?
        .get("id")?
        .as_dictionary()?
        .get("id")?
        .as_string()
}

/// The cached thumbnail path referenced by a gallery item, if any.
///
/// Items render their thumbnail from:
///
/// ```text
/// "thumbnail": { "image": { "url": { "relative": "file:///Users/me/.../Caches/<uuid>.png" } } }
/// ```
///
/// The URL is normalized to a plain [`PathBuf`]. Returns `None` when the item has no
/// thumbnail or a shape we do not recognize; the item is then simply removed without
/// touching any file.
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn gallery_item_thumbnail_path(item: &Value) -> Option<PathBuf> {
    let relative = item
        .as_dictionary()?
        .get("thumbnail")?
        .as_dictionary()?
        .get("image")?
        .as_dictionary()?
        .get("url")?
        .as_dictionary()?
        .get("relative")?
        .as_string()?;
    Some(normalize_file_url_path(relative))
}

/// Whether `path_str` refers to a file under `wallpapers_dir` which no longer
/// exists.
///
/// Must stay narrow: a path is stale only when it starts with Astra's own
/// `Wallpapers` directory **and** does not exist. That single predicate is what
/// keeps the currently-set wallpaper, other apps' photos, and system pictures safe,
/// so do not broaden it (for example, do not treat any missing file as stale).
///
/// Gallery paths are usually plain absolute paths, but entries written by older
/// versions or with unusual file names can be `file://` URLs and/or percent-encoded,
/// so both are normalized first.
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn is_stale_astra_path(path_str: &str, wallpapers_dir: &Path) -> bool {
    let path = normalize_file_url_path(path_str);
    path.starts_with(wallpapers_dir) && !path.exists()
}

/// Normalizes a path string into a filesystem path: strips a `file://` scheme
/// (including a `localhost` host) and percent-decodes `%XX` sequences.
///
/// Examples:
/// - `/Users/me/a.png` → `/Users/me/a.png`
/// - `file:///Users/me/my%20wall.png` → `/Users/me/my wall.png`
/// - `file://localhost/tmp/x.png` → `/tmp/x.png`
///
/// This is a string-level normalization, not a URL parser: anything that is not a
/// `%XX` escape is left as-is, and the result is only ever compared against
/// `wallpapers_dir` or used as a file path to delete (guarded by a
/// `starts_with(thumbnails_dir)` check at the call site).
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn normalize_file_url_path(path_str: &str) -> PathBuf {
    let without_scheme = path_str.strip_prefix("file://").unwrap_or(path_str);
    let decoded = percent_decode(without_scheme);
    let without_host = decoded
        .strip_prefix("localhost/")
        .map(|rest| format!("/{rest}"))
        .unwrap_or(decoded);
    PathBuf::from(without_host)
}

/// Decodes `%XX` hex sequences in `input`, leaving anything else (including
/// malformed sequences) untouched.
///
/// Deliberately lenient rather than strict: a malformed escape (a lone `%`, or
/// non-hex digits) is copied through unchanged instead of erroring, because this
/// runs on undocumented system data and must never fail a clean. Decoding produces
/// bytes and then uses `from_utf8_lossy`, so invalid UTF-8 cannot panic; it is only
/// used to build a path that is compared against, and deleted inside, known
/// directories.
// See the note on prune_wallpaper_gallery_file: macOS-only entry point.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && i + 2 < bytes.len()
            && let Ok(hex) = std::str::from_utf8(&bytes[i + 1..i + 3])
            && let Ok(byte) = u8::from_str_radix(hex, 16)
        {
            decoded.push(byte);
            i += 3;
            continue;
        }
        decoded.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

pub fn handle_generate_options(
    config: &Config,
    image_buf: &AstraImage,
    image: &Generator,
    no_save: bool,
    no_update: bool,
) -> Result<(), Box<dyn Error>> {
    // Handle options
    if !no_update {
        config.print_if_verbose(
            "NOTE: to update wallpaper, astra must save the image to astra_wallpapers folder.",
        );
        // Updating requires a saved image
        let saved_image_path = save_image(config, image, image_buf)?;
        // TODO: move verbose logs into OS implementations of update_wallpaper
        config.print_if_verbose("Updating wallpaper...");
        update_wallpaper(saved_image_path)?;
        config.print_if_verbose("Updated wallpaper");
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        let _ = save_image(config, image, image_buf)?;
    }
    Ok(())
}

/// Enum that specifies the color map generation algorithm
pub(super) enum Operator {
    Gradient,
}

/// Generates a color map based on the given parameters.
///
/// # Arguments
///
/// * `op` - The color map generation algorithm to use.
/// * `steps` - The number of color map entries to generate.
/// * `colors` - The colors to base the color map on.
///
/// # Returns
///
/// A vector of color map entries.
pub(super) fn create_color_map(op: Operator, steps: usize, colors: &[[u8; 3]]) -> Vec<[u8; 3]> {
    let mut color_map = Vec::with_capacity(steps);
    match op {
        Operator::Gradient => {
            if colors.len() == 1 {
                for _ in 0..steps {
                    color_map.push(colors[0]);
                }
            } else {
                let color_steps = (steps - 1) / (colors.len() - 1);
                for i in 0..steps {
                    let color_idx = (i as f64 / color_steps as f64).floor() as usize;
                    if color_idx == (colors.len() - 1) {
                        color_map.push(colors[color_idx]);
                    } else {
                        let new_color = mix_color(
                            colors[color_idx],
                            colors[color_idx + 1],
                            (i % color_steps) as f64 / color_steps as f64,
                        );
                        color_map.push(new_color);
                    }
                }
            }
        }
    }
    color_map
}

/// Interpolates between two colors to create a new color.
///
/// # Arguments
///
/// * `color1` - The first color.
/// * `color2` - The second color.
/// * `weight_color_2` - A value between 0 and 1 that determines how much of `color2` to include in the output.
///
/// # Returns
///
/// A new color that is a mix of `color1` and `color2`.
fn mix_color(color1: [u8; 3], color2: [u8; 3], weight_color_2: f64) -> [u8; 3] {
    let r = color1[0] as f64 * (1.0 - weight_color_2) + color2[0] as f64 * weight_color_2;
    let g = color1[1] as f64 * (1.0 - weight_color_2) + color2[1] as f64 * weight_color_2;
    let b = color1[2] as f64 * (1.0 - weight_color_2) + color2[2] as f64 * weight_color_2;
    [r as u8, g as u8, b as u8]
}

/// Saves the given image to a file in the desktop wallpaper folder.
///
/// The file is named using the current UNIX timestamp to ensure uniqueness.
/// The image is saved in PNG format.
///
/// # Arguments
///
/// * `config` - A reference to the `Config` struct.
/// * `prefix` - A string to prepend to the file name.
/// * `image` - A reference to the `ImageBuffer` containing the image to save.
///
/// # Returns
///
/// A `Result` containing the path to the saved image on success, or a
/// `WallpaperGeneratorError` on failure.
pub fn save_image(
    config: &Config,
    generator: &Generator,
    image: &AstraImage,
) -> Result<PathBuf, WallpaperGeneratorError> {
    config.print_if_verbose("Saving image to astra_wallpapers folder...");
    let mut save_path = create_wallpaper_folder()?;

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| WallpaperGeneratorError::OS(e.to_string()))?;

    save_path = save_path.join(format!("{}_{}.png", generator.prefix(), time.as_secs()));
    image
        .save(&save_path)
        .map_err(|_| WallpaperGeneratorError::ImageSave)?;
    config.print_if_verbose(
        format!(
            "Image saved to astra_wallpapers folder: {}",
            save_path.display()
        )
        .as_str(),
    );
    Ok(save_path)
}

/// Download an image from the URL into memory
///
/// # Retuns
///
/// A `Result` containing the image in memory in form of bytes on success
///
/// # Errors
///
/// * Network: If failure to donwload image OR convert into bytes
pub(super) fn download_image_to_memory(
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

/// Scales the range of the provided plane to generate a zoomed in image.
///
/// Given the original range of the plane, the center point of the
/// image to focus on, and a scale factor, this function returns the new
/// range of the plane and the start points of the image so that
/// the image will be centered and scaled accordingly while keeping
/// the focus point as the center of the image.
///
/// # Arguments
///
/// * `x_range` - The range of the original x-axis.
/// * `y_range` - The range of the original y-axis.
/// * `focus_pt` - The center point of the image to focus on.
/// * `scale_factor` - The scale factor to apply to the image.
///
/// # Returns
///
/// A tuple containing:
///   - the new range of the x-axis
///   - the new range of the y-axis
///   - the start point of the x-axis
///   -  the start point of the y-axis
pub(super) fn scale_image(
    x_range: f64,
    y_range: f64,
    focus_pt: (f64, f64),
    scale_factor: f64,
) -> (f64, f64, f64, f64) {
    // scale_factor 2 means halve the size of the image
    let scaled_x_range = x_range / scale_factor;
    let scaled_y_range = y_range / scale_factor;
    // get start points so if center of image then
    let x_start = focus_pt.0 - (scaled_x_range / 2.0);
    let y_start = focus_pt.1 - (scaled_y_range / 2.0);
    (scaled_x_range, scaled_y_range, x_start, y_start)
}

/// Calculates the average color of the image.
///
/// Reference https://stackoverflow.com/questions/649454/what-is-the-best-way-to-average-two-colors-that-define-a-linear-gradient
/// Formula:
///
/// `(r_avg, g_avg, b_avg) = (sqrt((R_0^2 + ... + R_n^2) / n), sqrt((G_0^2 + ... + G_n^2) / n), sqrt((B_0^2 + ... + B_n^2) / n))`
///
/// # Arguments
///
/// * `image` - The image to calculate the average color of.
///
/// # Returns
///
/// The average color of the image.
pub fn average_color(image: &AstraImage) -> Rgb<u8> {
    let mut rgb_avg = (0.0, 0.0, 0.0);
    image.pixels().for_each(|color| {
        let r = color[0] as f64;
        let g = color[1] as f64;
        let b = color[2] as f64;
        rgb_avg.0 += r.powi(2);
        rgb_avg.1 += g.powi(2);
        rgb_avg.2 += b.powi(2);
    });
    Rgb::from([
        (rgb_avg.0 / image.len() as f64).sqrt() as u8,
        (rgb_avg.1 / image.len() as f64).sqrt() as u8,
        (rgb_avg.2 / image.len() as f64).sqrt() as u8,
    ])
}

// --- Utils ---

// --- Errors ---
#[derive(Debug, PartialEq)]
pub enum WallpaperGeneratorError {
    ImageGeneration(String),
    ImageSave,
    Network(String),
    OS(String),
    Parse(String),
}

impl fmt::Display for WallpaperGeneratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WallpaperGeneratorError::ImageGeneration(msg) => {
                write!(f, "Image Generation Error: {}", msg)
            }
            WallpaperGeneratorError::ImageSave => {
                write!(f, "Failed to save image to file")
            }
            WallpaperGeneratorError::Network(msg) => {
                write!(f, "Network Error: {}", msg)
            }
            WallpaperGeneratorError::OS(msg) => {
                write!(f, "OS Error: {}", msg)
            }
            WallpaperGeneratorError::Parse(msg) => {
                write!(f, "Parse Error: {}", msg)
            }
        }
    }
}

impl Error for WallpaperGeneratorError {}
// --- Errors ---

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallpaper_timestamp_parses_astra_names() {
        assert_eq!(
            wallpaper_timestamp("spotlight_1640000000.png"),
            Some(1_640_000_000)
        );
        assert_eq!(
            wallpaper_timestamp("my_wall_1640000000.png"),
            Some(1_640_000_000)
        );
        // Legacy macOS slot files must still be deletable.
        assert_eq!(wallpaper_timestamp("astra_1.png"), Some(1));
        assert_eq!(wallpaper_timestamp("astra_2.png"), Some(2));
    }

    #[test]
    fn test_wallpaper_timestamp_rejects_other_names() {
        // These used to panic: shorter than 4 bytes, or a UTF-8 character split by
        // the old byte-index slice.
        for name in ["", "a", ".png", "a.png", "🙂.png"] {
            assert_eq!(
                wallpaper_timestamp(name),
                None,
                "unexpectedly parsed {name}"
            );
        }
        // A non-ASCII prefix is fine as long as the timestamp segment is numeric.
        assert_eq!(wallpaper_timestamp("é_1.png"), Some(1));

        for name in [
            "solid_123",     // missing the .png suffix
            "solid_abc.png", // non-numeric timestamp
            "solid_123.jpg", // wrong extension
            "solid.png",     // no timestamp
            "_.png",         // empty prefix and timestamp
            "solid_-5.png",  // negative
        ] {
            assert_eq!(
                wallpaper_timestamp(name),
                None,
                "unexpectedly parsed {name}"
            );
        }

        // Only the segment after the last underscore is the timestamp.
        assert_eq!(wallpaper_timestamp("solid_1_2.png"), Some(2));
    }

    #[test]
    fn test_scale_image() {
        let (x_range, y_range, x_start, y_start) = scale_image(10.0, 10.0, (0.0, 0.0), 2.0);
        assert_eq!(x_range, 5.0);
        assert_eq!(y_range, 5.0);
        assert_eq!(x_start, -2.5);
        assert_eq!(y_start, -2.5);
    }

    #[test]
    fn test_create_color_map_all_red() {
        let color_map = create_color_map(Operator::Gradient, 256, &[[255, 0, 0]]);
        assert_eq!(color_map.len(), 256);
        for color in color_map {
            assert_eq!(color, [255, 0, 0]);
        }
    }

    #[test]
    fn test_create_color_map_red_green() {
        let color_map = create_color_map(Operator::Gradient, 256, &[[255, 0, 0], [0, 255, 0]]);
        assert_eq!(color_map.len(), 256);
        assert_eq!(color_map[0], [255, 0, 0]);
        assert_eq!(color_map[255], [0, 255, 0]);
    }

    fn gallery_test_dirs() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let wallpapers_dir = dir.path().join("Wallpapers");
        let thumbnails_dir = dir.path().join("Thumbnails");
        std::fs::create_dir_all(&wallpapers_dir).unwrap();
        std::fs::create_dir_all(&thumbnails_dir).unwrap();
        (dir, wallpapers_dir, thumbnails_dir)
    }

    fn gallery_item(path: &Path, thumbnail: Option<&Path>) -> serde_json::Value {
        let mut item = serde_json::json!({
            "id": { "id": path.to_string_lossy() },
            "localizedName": path.file_stem().unwrap_or_default().to_string_lossy(),
            "disposability": { "removable": {} },
        });
        if let Some(thumbnail) = thumbnail {
            item["thumbnail"] = serde_json::json!({
                "image": {
                    "url": {
                        "relative": format!(
                            "file://{}",
                            thumbnail.to_string_lossy().replace(' ', "%20")
                        )
                    }
                }
            });
        }
        item
    }

    fn gallery_group(name: &str, items: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "id": { "id": name },
            "localizedName": name,
            "items": items,
        })
    }

    fn gallery_document(groups: Vec<serde_json::Value>) -> serde_json::Value {
        serde_json::json!({
            "osBuildVersion": "test",
            "viewModel": { "groups": groups, "isModificationDisabled": false },
        })
    }

    fn write_gallery(path: &Path, document: serde_json::Value) {
        let value: Value = serde_json::from_value(document).unwrap();
        value.to_file_binary(path).unwrap();
    }

    fn gallery_history_entry(path: &Path, thumbnail: Option<&Path>) -> Value {
        let mut entry = serde_json::json!({
            "originalURL": {
                "relative": format!("file://{}", path.to_string_lossy().replace(' ', "%20"))
            },
        });
        if let Some(thumbnail) = thumbnail {
            entry["copyURL"] = serde_json::json!({
                "relative": format!(
                    "file://{}",
                    thumbnail.to_string_lossy().replace(' ', "%20")
                )
            });
        }
        let value: Value = serde_json::from_value(entry).unwrap();
        let mut bytes = Vec::new();
        value.to_writer_binary(&mut bytes).unwrap();
        Value::Data(bytes)
    }

    fn write_gallery_history(path: &Path, entries: Vec<Value>) {
        let mut prefs = plist::Dictionary::new();
        prefs.insert(
            "ChoiceRequests.ImageFiles".to_string(),
            Value::Array(entries),
        );
        Value::Dictionary(prefs).to_file_binary(path).unwrap();
    }

    fn gallery_history_urls(path: &Path) -> Vec<String> {
        Value::from_file(path)
            .unwrap()
            .as_dictionary()
            .unwrap()
            .get("ChoiceRequests.ImageFiles")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|entry| {
                let Value::Data(bytes) = entry else {
                    return None;
                };
                let parsed = Value::from_reader(std::io::Cursor::new(bytes.as_slice())).ok()?;
                image_file_url(&parsed, "originalURL")
            })
            .collect()
    }

    fn gallery_item_paths(path: &Path) -> Vec<String> {
        let gallery = Value::from_file(path).unwrap();
        gallery
            .as_dictionary()
            .unwrap()
            .get("viewModel")
            .unwrap()
            .as_dictionary()
            .unwrap()
            .get("groups")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| {
                group
                    .as_dictionary()
                    .unwrap()
                    .get("items")
                    .unwrap()
                    .as_array()
                    .unwrap()
            })
            .filter_map(|item| gallery_item_path(item).map(str::to_string))
            .collect()
    }

    #[test]
    fn test_prune_wallpaper_gallery_removes_stale_astra_items_only() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();
        let stale = wallpapers_dir.join("julia_1600000000.png");

        let kept_thumb = thumbnails_dir.join("kept Uuid.png");
        let stale_thumb = thumbnails_dir.join("stale Uuid.png");
        let other_thumb = dir.path().join("elsewhere.png");
        std::fs::write(&kept_thumb, b"").unwrap();
        std::fs::write(&stale_thumb, b"").unwrap();
        std::fs::write(&other_thumb, b"").unwrap();

        let gallery_path = dir.path().join("image-desktop");
        write_gallery(
            &gallery_path,
            gallery_document(vec![gallery_group(
                "YourPhotosGroup",
                vec![
                    gallery_item(&kept, Some(&kept_thumb)),
                    gallery_item(&stale, Some(&stale_thumb)),
                    // A stale item whose thumbnail is outside the cache directory
                    // must not cause that file to be deleted.
                    gallery_item(&wallpapers_dir.join("missing.png"), Some(&other_thumb)),
                    // Non-Astra and synthetic items are always kept.
                    gallery_item(
                        Path::new("/System/Library/Desktop Pictures/Mac Purple.heic"),
                        None,
                    ),
                    serde_json::json!({ "id": { "id": "add_photo" } }),
                ],
            )]),
        );

        let config = Config::new(false);
        let removed = prune_wallpaper_gallery_file(
            &config,
            &gallery_path,
            &wallpapers_dir,
            Some(&thumbnails_dir),
        );
        assert_eq!(removed, 2);
        assert!(!stale_thumb.exists());
        assert!(kept_thumb.exists());
        assert!(other_thumb.exists());

        let paths = gallery_item_paths(&gallery_path);
        assert_eq!(paths.len(), 3);
        assert!(paths.contains(&kept.to_string_lossy().into_owned()));
        assert!(paths.contains(&"/System/Library/Desktop Pictures/Mac Purple.heic".to_string()));
        assert!(paths.contains(&"add_photo".to_string()));
    }

    #[test]
    fn test_prune_wallpaper_gallery_removes_across_groups() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();

        let gallery_path = dir.path().join("image-desktop");
        write_gallery(
            &gallery_path,
            gallery_document(vec![
                gallery_group(
                    "YourPhotosGroup",
                    vec![gallery_item(&wallpapers_dir.join("a_1.png"), None)],
                ),
                gallery_group(
                    "AnotherGroup",
                    vec![
                        gallery_item(&wallpapers_dir.join("b_2.png"), None),
                        gallery_item(&kept, None),
                    ],
                ),
            ]),
        );

        let config = Config::new(false);
        assert_eq!(
            prune_wallpaper_gallery_file(
                &config,
                &gallery_path,
                &wallpapers_dir,
                Some(&thumbnails_dir)
            ),
            2
        );
        let paths = gallery_item_paths(&gallery_path);
        assert_eq!(paths, vec![kept.to_string_lossy().into_owned()]);
    }

    #[test]
    fn test_prune_wallpaper_gallery_leaves_clean_gallery_untouched() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();

        let gallery_path = dir.path().join("image-desktop");
        write_gallery(
            &gallery_path,
            gallery_document(vec![gallery_group(
                "YourPhotosGroup",
                vec![gallery_item(&kept, None)],
            )]),
        );
        let original = std::fs::read(&gallery_path).unwrap();

        let config = Config::new(false);
        assert_eq!(
            prune_wallpaper_gallery_file(
                &config,
                &gallery_path,
                &wallpapers_dir,
                Some(&thumbnails_dir)
            ),
            0
        );
        assert_eq!(std::fs::read(&gallery_path).unwrap(), original);
    }

    #[test]
    fn test_prune_wallpaper_gallery_missing_and_invalid() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let config = Config::new(false);

        let missing = dir.path().join("does-not-exist");
        assert_eq!(
            prune_wallpaper_gallery_file(&config, &missing, &wallpapers_dir, Some(&thumbnails_dir)),
            0
        );
        assert!(!missing.exists());

        let invalid = dir.path().join("invalid");
        std::fs::write(&invalid, b"not a plist").unwrap();
        assert_eq!(
            prune_wallpaper_gallery_file(&config, &invalid, &wallpapers_dir, Some(&thumbnails_dir)),
            0
        );
        assert_eq!(std::fs::read(&invalid).unwrap(), b"not a plist");
    }

    #[test]
    fn test_normalize_file_url_path() {
        let dir = tempfile::tempdir().unwrap();
        let with_space = dir.path().join("my wall.png");
        let url = format!(
            "file://{}",
            with_space.to_string_lossy().replace(' ', "%20")
        );
        assert_eq!(normalize_file_url_path(&url), with_space);
        assert_eq!(
            normalize_file_url_path("file://localhost/tmp/x.png"),
            PathBuf::from("/tmp/x.png")
        );
        assert_eq!(
            normalize_file_url_path("/tmp/plain.png"),
            PathBuf::from("/tmp/plain.png")
        );
        assert_eq!(percent_decode("100%25 and %zz"), "100% and %zz");
    }

    #[test]
    fn test_wallpaper_gallery_has_stale_entries() {
        let (dir, wallpapers_dir, _thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();

        let gallery_path = dir.path().join("image-desktop");
        let history_path = dir.path().join("prefs.plist");
        write_gallery(
            &gallery_path,
            gallery_document(vec![gallery_group(
                "YourPhotosGroup",
                vec![gallery_item(&kept, None)],
            )]),
        );
        write_gallery_history(&history_path, vec![gallery_history_entry(&kept, None)]);
        assert!(!wallpaper_gallery_has_stale_entries(
            Some(&gallery_path),
            Some(&history_path),
            &wallpapers_dir
        ));

        // A missing or unreadable file must not be reported as stale: there is
        // nothing to prune, so the wallpaper services should not be stopped.
        assert!(!wallpaper_gallery_has_stale_entries(
            Some(&dir.path().join("missing")),
            Some(&dir.path().join("missing")),
            &wallpapers_dir
        ));

        write_gallery(
            &gallery_path,
            gallery_document(vec![gallery_group(
                "YourPhotosGroup",
                vec![gallery_item(&wallpapers_dir.join("a.png"), None)],
            )]),
        );
        assert!(wallpaper_gallery_has_stale_entries(
            Some(&gallery_path),
            None,
            &wallpapers_dir
        ));

        write_gallery(
            &gallery_path,
            gallery_document(vec![gallery_group(
                "YourPhotosGroup",
                vec![gallery_item(&kept, None)],
            )]),
        );
        write_gallery_history(
            &history_path,
            vec![gallery_history_entry(&wallpapers_dir.join("b.png"), None)],
        );
        assert!(wallpaper_gallery_has_stale_entries(
            None,
            Some(&history_path),
            &wallpapers_dir
        ));
    }

    #[test]
    fn test_prune_wallpaper_extension_image_files_removes_stale_only() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();
        let stale = wallpapers_dir.join("julia_1600000000.png");
        let kept_thumb = thumbnails_dir.join("kept.png");
        let stale_thumb = thumbnails_dir.join("stale.png");
        let other_thumb = dir.path().join("elsewhere.png");
        for file in [&kept_thumb, &stale_thumb, &other_thumb] {
            std::fs::write(file, b"").unwrap();
        }

        let prefs_path = dir.path().join("prefs.plist");
        write_gallery_history(
            &prefs_path,
            vec![
                gallery_history_entry(&kept, Some(&kept_thumb)),
                gallery_history_entry(&stale, Some(&stale_thumb)),
                gallery_history_entry(&wallpapers_dir.join("missing.png"), Some(&other_thumb)),
                gallery_history_entry(Path::new("/somewhere/else/old.png"), None),
            ],
        );

        let config = Config::new(false);
        let removed = prune_wallpaper_extension_image_files(
            &config,
            &prefs_path,
            &wallpapers_dir,
            Some(&thumbnails_dir),
        );
        assert_eq!(removed, 2);
        assert!(!stale_thumb.exists());
        assert!(kept_thumb.exists());
        assert!(other_thumb.exists());

        let urls = gallery_history_urls(&prefs_path);
        assert_eq!(urls.len(), 2);
        assert!(urls.iter().any(|url| url.ends_with("solid_1700000000.png")));
        assert!(urls.iter().any(|url| url.ends_with("old.png")));
    }

    #[test]
    fn test_prune_wallpaper_extension_image_files_leaves_clean_history_untouched() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let kept = wallpapers_dir.join("solid_1700000000.png");
        std::fs::write(&kept, b"").unwrap();

        let prefs_path = dir.path().join("prefs.plist");
        write_gallery_history(&prefs_path, vec![gallery_history_entry(&kept, None)]);
        let original = std::fs::read(&prefs_path).unwrap();

        let config = Config::new(false);
        assert_eq!(
            prune_wallpaper_extension_image_files(
                &config,
                &prefs_path,
                &wallpapers_dir,
                Some(&thumbnails_dir)
            ),
            0
        );
        assert_eq!(std::fs::read(&prefs_path).unwrap(), original);
    }

    #[test]
    fn test_prune_wallpaper_extension_image_files_missing_and_invalid() {
        let (dir, wallpapers_dir, thumbnails_dir) = gallery_test_dirs();
        let config = Config::new(false);

        let missing = dir.path().join("missing-prefs");
        assert_eq!(
            prune_wallpaper_extension_image_files(
                &config,
                &missing,
                &wallpapers_dir,
                Some(&thumbnails_dir)
            ),
            0
        );
        assert!(!missing.exists());

        let invalid = dir.path().join("invalid-prefs");
        std::fs::write(&invalid, b"not a plist").unwrap();
        assert_eq!(
            prune_wallpaper_extension_image_files(
                &config,
                &invalid,
                &wallpapers_dir,
                Some(&thumbnails_dir)
            ),
            0
        );
        assert_eq!(std::fs::read(&invalid).unwrap(), b"not a plist");
    }
}
// --- Tests ---
