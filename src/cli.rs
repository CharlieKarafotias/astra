use super::Color;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

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
        /// Delete images older than X days
        older_than: Option<u64>,
        #[arg(short, long, default_value_t = false)]
        /// Deletes all images and the "astra_wallpapers" directory
        directory: bool,
    },
    /// Generates a new wallpaper
    Generate {
        /// The type of image to generate
        #[command(subcommand)]
        image: ImageType,
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
    }
}

#[derive(Clone, Debug, Subcommand)]
pub enum ImageType {
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

#[derive(Clone, Debug, Subcommand)]
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

pub enum Mode {
    Solid(SolidMode),
}

pub struct Config {
    verbose: bool,
}

impl Config {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn print_if_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }
}
