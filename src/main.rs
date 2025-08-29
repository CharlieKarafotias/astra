mod cli;
mod config;
mod constants;
mod os_implementations;
mod wallpaper_generators;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands, Generator};
use config::{Config, Generators};
use rand::random_range;
use wallpaper_generators::{
    Color, delete_wallpapers, generate_bing_spotlight, generate_julia_set, generate_solid_color,
    handle_generate_options,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config = Config::new(cli.verbose);

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
                // TODO v1.1.0 - use the os_impl mod and setup for each os instead of this
                if let Ok(editor) = std::env::var("EDITOR") {
                    std::process::Command::new(editor)
                        .arg(Config::config_path())
                        .status()
                        .map_err(|e| {
                            format!("failed to open configuration file: {}", e.to_string())
                        })?;
                }
                // TODO v1.1.0: if not found, probably need to make a WARN here and default to showing path instead
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
            // Since 'astra' was called, respect user config
            config.respect_user_config = true;
            let generators = config
                .generators()
                .as_ref()
                .map(|generators| generators.to_vec())
                .unwrap_or(Generators::ALL_GENERATORS.to_vec());

            // TODO: add in automatic call of astra on cron schedule
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
