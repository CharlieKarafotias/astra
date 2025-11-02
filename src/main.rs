mod cli;
mod configuration;
mod constants;
mod os_implementations;
mod themes;
mod wallpaper_generators;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use cli::{Cli, Commands, Generator};
use configuration::{Config, Frequency, Generators};
use os_implementations::open_editor;
use rand::random_range;
use wallpaper_generators::{
    Color, delete_wallpapers, generate_bing_spotlight, generate_julia_set, generate_solid_color,
    handle_generate_options,
};

use crate::os_implementations::handle_frequency;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut config = Config::new(cli.verbose);

    // TODO: Errors coming out in strange format. Fix this so its standardized (Error: ParseError("invalid...")) looks weird
    match cli.command {
        Some(Commands::Clean {
            older_than,
            directory,
        }) => {
            if let Some(older_than) = older_than {
                let frequency = Frequency::new(older_than.as_str())?;
                delete_wallpapers(&config, false, directory, Some(&frequency))?;
            } else {
                config.print_if_verbose("Deleting all images...");
                delete_wallpapers(&config, true, directory, None)?;
            }
        }
        Some(Commands::Config { open }) => {
            config.print_if_verbose("Opening configuration file...");
            Config::create_config_file_if_not_exists(&config)?;
            if open {
                open_editor(&config, Config::config_path())?;
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

            if let Some(auto_clean_frequency) = config.auto_clean() {
                config.print_if_verbose(
                    format!(
                        "Auto clean enabled - cleaning images older than {}",
                        auto_clean_frequency
                    )
                    .as_str(),
                );
                delete_wallpapers(&config, false, false, config.auto_clean())?;
            }

            let generators = config
                .generators()
                .as_ref()
                .map(|generators| generators.to_vec())
                .unwrap_or(Generators::ALL_GENERATORS.to_vec());

            handle_frequency(&config)?;

            let index = random_range(0..generators.len());
            let image_type = &generators[index];
            let image_buf = image_type.with_default_mode(&config)?;
            handle_generate_options(&config, &image_buf, image_type, false, false)?;
        }
    };
    Ok(())
}
