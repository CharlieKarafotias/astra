use super::Color;
use crate::{
    configuration::Config,
    wallpaper_generators::{
        AstraImage, WallpaperGeneratorError, generate_bing_spotlight, generate_julia_set,
        generate_solid_color,
    },
};
use clap::{Parser, Subcommand};
use clap_complete::Shell;
use std::str::FromStr;

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    /// Subcommands
    pub(crate) command: Option<Commands>,
    #[arg(short, long)]
    /// Verbose output
    pub(crate) verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Deletes images from "astra_wallpapers" folder (deletes all images by default)
    Clean {
        #[arg(short, long)]
        /// Delete images older than a frequency (e.g. 1s, 2m, 3h, 4d, 5w, 6m, 7y)
        older_than: Option<String>,
        #[arg(short, long, default_value_t = false)]
        /// Deletes all images and the "astra_wallpapers" directory
        directory: bool,
    },
    /// Return path to configuration file (creates config first if it doesn't exist)
    Config {
        #[arg(short, long)]
        /// Open the configuration file in the default text editor
        open: bool,
    },
    /// Generates a new wallpaper
    Generate {
        /// The type of image to generate
        #[command(subcommand)]
        image: Generator,
        #[arg(long)]
        /// Skip saving the image to the "astra_wallpapers" folder.
        no_save: bool,
        #[arg(long)]
        /// Skip updating current desktop wallpaper to generated image
        no_update: bool,
    },
    /// Generate shell completion scripts
    GenerateCompletions {
        /// The shell to generate completion scripts for
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum Generator {
    /// Sets wallpaper to a randomly generated Julia Set
    Julia,
    /// Sets wallpaper to a solid color
    Solid {
        #[command(subcommand)]
        mode: SolidMode,
    },
    /// Sets wallpaper to one of Bing's daily Spotlight images
    Spotlight,
}

impl FromStr for Generator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "julia" => Ok(Generator::Julia),
            "spotlight" => Ok(Generator::Spotlight),
            "solid" => Ok(Generator::Solid {
                mode: SolidMode::Random,
            }),
            _ => Err(format!("Unknown generator type: {}", s)),
        }
    }
}

impl Generator {
    pub fn with_default_mode(
        &self,
        config: &Config,
    ) -> Result<AstraImage, WallpaperGeneratorError> {
        match self {
            Generator::Julia => generate_julia_set(config),
            Generator::Solid { mode } => generate_solid_color(config, mode),
            Generator::Spotlight => generate_bing_spotlight(config),
        }
    }

    pub fn prefix(&self) -> &str {
        match self {
            Generator::Julia => "julia",
            Generator::Solid { mode: _ } => "solid",
            Generator::Spotlight => "spotlight",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum SolidMode {
    /// Use a pre-defined color
    Color {
        /// Color name (see all options with `--help`)
        #[arg(value_enum)]
        name: Color,
    },
    /// Use a random color
    Random,
    /// Use a custom color by RGB value
    Rgb {
        /// Red component (0-255)
        r: u8,
        /// Green component (0-255)
        g: u8,
        /// Blue component (0-255)
        b: u8,
    },
}
