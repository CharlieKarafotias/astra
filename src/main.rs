mod cli;
mod config;
mod constants;
mod os_implementations;
mod wallpaper_generators;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands, Generator};
use config::{Config, Generators};
use os_implementations::update_wallpaper;
use rand::random_range;
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
        Some(Commands::Config { open }) => {
            config.print_if_verbose("Opening configuration file...");
            Config::create_config_file_if_not_exists(&config)?;
            if open {
                if let Ok(editor) = std::env::var("EDITOR") {
                    std::process::Command::new(editor)
                        .arg(Config::config_path())
                        .status()
                        .map_err(|e| {
                            format!("failed to open configuration file: {}", e.to_string())
                        })?;
                }
            } else {
                println!("{}", Config::config_path().display());
            }
        }
        Some(Commands::Generate {
            image,
            no_save,
            no_update,
        }) => {
            config.print_if_verbose(format!("Generating image of type: {:?}...", &image).as_str());
            let image_buf = match &image {
                Generator::Julia => generate_julia_set(&config),
                Generator::Solid { mode } => generate_solid_color(&config, mode),
                Generator::Spotlight => generate_bing_spotlight(&config),
            }?;
            handle_generate_options(&config, &image_buf, &image, no_save, no_update)?;
        }
        Some(Commands::GenerateCompletions { shell }) => {
            generate(shell, &mut Cli::command(), "astra", &mut std::io::stdout());
        }
        None => {
            // TODO - v1.1.0: Update this logic to respect configuration file if present. Need to add function to read config file.
            // References:
            // - https://docs.rs/directories/latest/directories/
            // - https://docs.rs/regex/latest/regex/
            // - JSON serde and serde for turning config to struct
            // Default to generate a random image
            // TODO: Ideally, there's preferences for types of images user likes and pref for how often to change wallpaper
            // I think in install directions, should have option to call astra on startup of terminal and auto check if wallpaper needs to be changed based on some preference of how often
            let generators = config
                .generators()
                .as_ref()
                .map(|generators| generators.to_vec())
                .unwrap_or(Generators::ALL_GENERATORS.to_vec());
            // TODO: use this to setup automatic wallpaper refresh
            let _frequency = config.frequency();

            let index = random_range(0..generators.len());
            let image_type = &generators[index];
            let image_buf = image_type.with_default_mode(&config)?;
            handle_generate_options(&config, &image_buf, image_type, false, false)?;
        }
    };

    Ok(())
}

// TODO: move to astra_logic module
fn save_image_to_astra_folder(
    config: &Config,
    image: &Generator,
    image_buf: &AstraImage,
) -> Result<PathBuf, WallpaperGeneratorError> {
    let prefix = match image {
        Generator::Julia => "julia",
        Generator::Solid { .. } => "solid",
        Generator::Spotlight => "spotlight",
    };
    let path = save_image(&config, prefix, &image_buf)?;
    Ok(path)
}

// TODO: move to astra_logic module
fn handle_generate_options(
    config: &Config,
    image_buf: &AstraImage,
    image: &Generator,
    no_save: bool,
    no_update: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Handle options
    if !no_update {
        config.print_if_verbose(
            "NOTE: to update wallpaper, astra must save the image to astra_wallpapers folder.",
        );
        // Updating requires a saved image
        let saved_image_path = save_image_to_astra_folder(config, image, image_buf)?;
        // TODO: move verbose logs into OS implementations of update_wallpaper
        config.print_if_verbose("Updating wallpaper...");
        update_wallpaper(saved_image_path)?;
        config.print_if_verbose("Updated wallpaper");
    }
    // If no_update == false, we already saved the image as its required to update wallpaper
    if no_update && !no_save {
        let _ = save_image_to_astra_folder(&config, &image, image_buf)?;
    }
    Ok(())
}
