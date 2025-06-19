mod cli;
mod os_implementations;
mod wallpaper_generators;

use clap::Parser;
use cli::{Cli, Commands, ImageType};
use os_implementations::update_wallpaper;
use rand::random_range;
use std::path::PathBuf;
use wallpaper_generators::{
    AstraImage, WallpaperGeneratorError, delete_wallpapers, generate_bing_spotlight,
    generate_julia_set, save_image,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let verbose = cli.verbose;
    
    match cli.command {
        Some(Commands::Clean {
            older_than,
            directory,
        }) => {
            if verbose {
                println!("Deleting images older than {} days", older_than.unwrap_or(0));
            }
            if let Some(older_than) = older_than {
                delete_wallpapers(false, directory, Some(older_than), verbose)?;
            } else {
                // Delete all images and if directory is true, delete the "astra_wallpapers" folder
                delete_wallpapers(true, directory, None, verbose)?;
            }
        }
        Some(Commands::Generate {
            image,
            no_save,
            no_update,
        }) => {
            if verbose {
                println!("Generating image of type: {:?}...", image);
            }
            let image_buf = match image {
                ImageType::Spotlight => generate_bing_spotlight(verbose),
                ImageType::Julia => generate_julia_set(verbose),
            }?;
            handle_generate_options(image_buf, image.clone(), no_save, no_update, verbose)?;
        }
        None => {
            // Default to generate a random image
            // TODO: Ideally, there's preferences for types of images user likes and pref for how often to change wallpaper
            // I think in install directions, should have option to call astra on startup of terminal and auto check if wallpaper needs to be changed based on some preference of how often
            let generators: [(
                fn(verbose: bool) -> Result<AstraImage, WallpaperGeneratorError>,
                ImageType,
            ); 2] = [
                (generate_bing_spotlight, ImageType::Spotlight),
                (generate_julia_set, ImageType::Julia),
            ];
            let index = random_range(0..generators.len());
            let image_buf = generators[index].0(verbose)?;
            let image_type = generators[index].1.clone();
            handle_generate_options(image_buf, image_type, false, false, verbose)?;
        }
    };

    Ok(())
}

fn save_image_to_astra_folder(
    image: &ImageType,
    image_buf: &AstraImage,
    verbose: bool
) -> Result<PathBuf, WallpaperGeneratorError> {
    let prefix = match image {
        ImageType::Spotlight => "spotlight",
        ImageType::Julia => "julia",
    };
    if verbose {
        println!("Saving image to astra_wallpapers folder...");
    }
    let path = save_image(prefix, &image_buf)?;
    if verbose {
        println!("Saved image to astra_wallpapers folder");
    }
    Ok(path)
}

fn handle_generate_options(
    image_buf: AstraImage,
    image: ImageType,
    no_save: bool,
    no_update: bool,
    verbose: bool
) -> Result<(), Box<dyn std::error::Error>> {
    // Handle options
    if !no_update {
        if verbose {
            println!("Saving image to astra_wallpapers folder (this is required to update wallpaper)");
        }
        // Updating requires a saved image
        let saved_image_path = save_image_to_astra_folder(&image, &image_buf, verbose)?;
        if verbose {
            println!("Updating wallpaper...");
        }
        update_wallpaper(saved_image_path)?;
        if verbose {
            println!("Updated wallpaper");
        }
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        if verbose {
            println!("Saving image to astra_wallpapers folder...");
        }
        let _ = save_image_to_astra_folder(&image, &image_buf, verbose)?;
        if verbose {
            println!("Image saved to astra_wallpapers folder");
        }
    }
    Ok(())
}
