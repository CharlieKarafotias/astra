mod cli;
mod os_implementations;
mod wallpaper_generators;

use clap::Parser;
use cli::{Cli, Commands, Config, ImageType, Mode, SolidMode};
use os_implementations::update_wallpaper;
use rand::random_range;
use std::fmt::format;
use std::path::PathBuf;
use wallpaper_generators::{
    AstraImage, Color, WallpaperGeneratorError, delete_wallpapers, generate_bing_spotlight,
    generate_julia_set, generate_solid_color, save_image,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = Config::new(cli.verbose);

    match cli.command {
        Some(Commands::Clean {
            older_than,
            directory,
        }) => {
            config.print_if_verbose(
                format!(
                    "Deleting images older than {} days",
                    older_than.unwrap_or(0)
                )
                .as_str(),
            );
            if let Some(older_than) = older_than {
                delete_wallpapers(&config, false, directory, Some(older_than))?;
            } else {
                // Delete all images and if directory is true, delete the "astra_wallpapers" folder
                delete_wallpapers(&config, true, directory, None)?;
            }
        }
        Some(Commands::Generate {
            image,
            no_save,
            no_update,
        }) => {
            config.print_if_verbose(format!("Generating image of type: {:?}...", &image).as_str());
            let image_buf = match &image {
                ImageType::Julia => generate_julia_set(&config, None),
                ImageType::Solid { mode } => {
                    generate_solid_color(&config, Some(Mode::Solid(mode.clone())))
                } // TODO: improve this
                ImageType::Spotlight => generate_bing_spotlight(&config, None),
            }?;
            handle_generate_options(&config, image_buf, image.clone(), no_save, no_update)?;
        }
        None => {
            // Default to generate a random image
            // TODO: Ideally, there's preferences for types of images user likes and pref for how often to change wallpaper
            // I think in install directions, should have option to call astra on startup of terminal and auto check if wallpaper needs to be changed based on some preference of how often
            // TODO: need to refactor so function is fn(generator_options: GeneratorOptions, astra_ctx: AstraContext) -> Result<AstraImage, WallpaperGeneratorError>
            let generators: [(
                fn(
                    config: &Config,
                    mode: Option<Mode>,
                ) -> Result<AstraImage, WallpaperGeneratorError>,
                ImageType,
            ); 3] = [
                (generate_julia_set, ImageType::Julia),
                (
                    generate_solid_color,
                    ImageType::Solid {
                        mode: SolidMode::Random,
                    },
                ),
                (generate_bing_spotlight, ImageType::Spotlight),
            ];
            let index = random_range(0..generators.len());
            let image_type = generators[index].1.clone();
            let image_buf = match &image_type {
                ImageType::Solid { mode } => {
                    generate_solid_color(&config, Some(Mode::Solid(mode.clone())))?
                }
                ImageType::Julia => generate_julia_set(&config, None)?,
                ImageType::Spotlight => generate_bing_spotlight(&config, None)?,
            };
            handle_generate_options(&config, image_buf, image_type, false, false)?;
        }
    };

    Ok(())
}

// TODO: move to astra_logic module
fn save_image_to_astra_folder(
    config: &Config,
    image: &ImageType,
    image_buf: &AstraImage,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let prefix = match image {
        ImageType::Julia => "julia",
        ImageType::Solid { .. } => "solid",
        ImageType::Spotlight => "spotlight",
    };
    let path = save_image(&config, prefix, &image_buf)?;
    Ok(path)
}

// TODO: move to astra_logic module
fn handle_generate_options(
    config: &Config,
    image_buf: AstraImage,
    image: ImageType,
    no_save: bool,
    no_update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Handle options
    if !no_update {
        config.print_if_verbose(
            "NOTE: to update wallpaper, astra must save the image to astra_wallpapers folder.",
        );
        // Updating requires a saved image
        let saved_image_path = save_image_to_astra_folder(&config, &image, &image_buf)?;
        // TODO: move verbose logs into OS implementations of update_wallpaper
        config.print_if_verbose("Updating wallpaper...");
        update_wallpaper(saved_image_path)?;
        config.print_if_verbose("Updated wallpaper");
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        let _ = save_image_to_astra_folder(&config, &image, &image_buf)?;
    }
    Ok(())
}
