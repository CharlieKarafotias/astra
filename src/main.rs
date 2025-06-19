mod cli;
mod os_implementations;
mod wallpaper_generators;

use clap::Parser;
use cli::{Cli, Commands, ImageType};
use os_implementations::update_wallpaper;
use rand::random_range;
use std::path::PathBuf;
use wallpaper_generators::{
    AstraImage, WallpaperGeneratorError, generate_bing_spotlight, generate_julia_set, save_image,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let (image_buf, image, no_save, no_update) = match &cli.command {
        Some(Commands::Generate {
            image,
            no_save,
            no_update,
        }) => {
            // Generate correct image
            let image_buf = match image {
                ImageType::Spotlight => generate_bing_spotlight(),
                ImageType::Julia => generate_julia_set(),
            }?;
            (image_buf, image.clone(), *no_save, *no_update)
        }
        Some(Commands::Clean {
            older_than,
            directory,
        }) => todo!("Implement clean command"),
        _ => {
            // TODO: Ideally, there's preferences for types of images user likes and pref for how often to change wallpaper
            // I think in install directions, should have option to call astra on startup of terminal and auto check if wallpaper needs to be changed based on some preference of how often
            let generators: [(
                fn() -> Result<AstraImage, WallpaperGeneratorError>,
                ImageType,
            ); 2] = [
                (generate_bing_spotlight, ImageType::Spotlight),
                (generate_julia_set, ImageType::Julia),
            ];
            let index = random_range(0..generators.len());
            let image_buf = generators[index].0()?;
            let image_type = generators[index].1.clone();
            (image_buf, image_type, false, false)
        }
    };

    // Handle options
    if !no_update {
        // Updating requires a saved image
        let saved_image_path = save_image_to_astra_folder(&image, &image_buf)?;
        update_wallpaper(saved_image_path)?;
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        let _ = save_image_to_astra_folder(&image, &image_buf)?;
    }
    Ok(())
}

fn save_image_to_astra_folder(
    image: &ImageType,
    image_buf: &AstraImage,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let prefix = match image {
        ImageType::Spotlight => "spotlight",
        ImageType::Julia => "julia",
    };
    save_image(prefix, &image_buf)
}
