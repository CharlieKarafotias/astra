use super::Color;
use crate::{
    configuration::Config,
    wallpaper_generators::{
        ApodDate, AstraImage, WallpaperGeneratorError, generate_bing_spotlight, generate_julia_set,
        generate_nasa_apod, generate_sdxl, generate_solid_color, parse_yymmdd,
        sdxl_can_run_on_this_machine,
    },
};
use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;
use std::path::PathBuf;
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
    /// Return path to configuration file (creates it and its JSON Schema if needed)
    Config {
        #[arg(short, long)]
        /// Open the configuration file in the default text editor
        open: bool,
        #[arg(short, long, conflicts_with = "open")]
        /// Print the JSON schema for the configuration file
        schema: bool,
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
    /// Manage the local text-to-image (SDXL Turbo) model files
    Models {
        #[command(subcommand)]
        command: ModelCommand,
    },
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum ModelCommand {
    /// Download the SDXL Turbo model files into the Hugging Face cache
    Download,
    /// List the model files and whether they are already cached
    List,
}

/// Options for the local, pure-Rust SDXL Turbo generator.
#[derive(Clone, Debug, Default, PartialEq, Args)]
pub struct SdxlOptions {
    /// Text prompt. If omitted, one is generated from `sdxl_gen.characteristics`
    #[arg(long)]
    pub prompt: Option<String>,
    /// Negative prompt (ignored by SDXL Turbo at the default guidance scale of 0)
    #[arg(long)]
    pub negative_prompt: Option<String>,
    /// Output width in pixels (multiple of 8, max 2048). Defaults to the screen width fitted to SDXL
    #[arg(long)]
    pub width: Option<u32>,
    /// Output height in pixels (multiple of 8, max 2048). Defaults to the screen height fitted to SDXL
    #[arg(long)]
    pub height: Option<u32>,
    /// Number of denoising steps (1-10; SDXL Turbo needs 1-4)
    #[arg(long)]
    pub steps: Option<u32>,
    /// Classifier-free guidance scale (SDXL Turbo is distilled for 0.0)
    #[arg(long)]
    pub guidance_scale: Option<f64>,
    /// Random seed. The same prompt, size, steps and seed produce the same image
    #[arg(long)]
    pub seed: Option<u64>,
    /// Also write the generated image to this path
    #[arg(long)]
    pub output: Option<PathBuf>,
    /// Faster, lower-resolution preview: fit the screen to ~0.6 MP and use 3 steps
    #[arg(long)]
    pub fast: bool,
    /// Validate the prompt, device and dimensions without downloading or running diffusion
    #[arg(long)]
    pub dry_run: bool,
    /// Allow running on the CPU (slow and memory hungry)
    #[arg(long)]
    pub allow_cpu: bool,
}

#[derive(Clone, Debug, PartialEq, Subcommand)]
pub enum Generator {
    /// Sets wallpaper to a randomly generated Julia Set
    Julia,
    /// Sets wallpaper to one of NASA's Astronomy Pictures of the Day (if no args passed, defaults
    /// to today's image)
    NasaAPOD {
        /// Specify specific day (format: YYMMDD)
        #[arg(value_parser = parse_yymmdd)]
        date: Option<ApodDate>,
        /// Use a random picture of the day instead of a specific date
        #[arg(long, conflicts_with = "date")]
        random: bool,
    },
    /// Sets wallpaper to a solid color
    Solid {
        #[command(subcommand)]
        mode: SolidMode,
    },
    /// Sets wallpaper to one of Bing's daily Spotlight images
    Spotlight,
    /// Generates a photorealistic wallpaper locally with SDXL Turbo (pure Rust)
    Sdxl(SdxlOptions),
}

impl FromStr for Generator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "julia" => Ok(Generator::Julia),
            "nasa_apod" => Ok(Generator::NasaAPOD {
                date: None,
                random: false,
            }),
            "spotlight" => Ok(Generator::Spotlight),
            "solid" => Ok(Generator::Solid {
                mode: SolidMode::Random,
            }),
            "sdxl" => Ok(Generator::Sdxl(SdxlOptions::default())),
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
            Generator::NasaAPOD { date, random } => generate_nasa_apod(config, date, *random),
            Generator::Solid { mode } => generate_solid_color(config, mode),
            Generator::Spotlight => generate_bing_spotlight(config),
            Generator::Sdxl(options) => generate_sdxl(config, options),
        }
    }

    pub fn prefix(&self) -> &str {
        match self {
            Generator::Julia => "julia",
            Generator::NasaAPOD { date: _, random: _ } => "nasa_apod",
            Generator::Solid { mode: _ } => "solid",
            Generator::Spotlight => "spotlight",
            Generator::Sdxl(_) => "sdxl",
        }
    }

    /// Whether this generator can run unattended (auto mode) on this machine.
    pub fn is_available(&self) -> bool {
        match self {
            Generator::Sdxl(_) => sdxl_can_run_on_this_machine(),
            _ => true,
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
