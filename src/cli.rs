use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    /// Subcommands
    pub(crate) command: Option<Commands>,
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
        image: ImageType,
        #[arg(long)]
        /// Skip saving the image to the "astra_wallpapers" folder.
        no_save: bool,
        #[arg(long)]
        /// Skip updating current desktop wallpaper to generated image
        no_update: bool,
    },
}

#[derive(Clone, ValueEnum)]
pub enum ImageType {
    /// Sets wallpaper to one of Bing's daily Spotlight images
    Spotlight,
    /// Sets wallpaper to a randomly generated Julia Set
    Julia,
}
