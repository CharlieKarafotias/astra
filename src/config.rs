use super::constants::{APPLICATION, ORGANIZATION, QUALIFIER};
use directories::ProjectDirs;
use serde::Deserialize;
use serde_json;
use std::fs;

pub struct Config {
    // From CLI options
    verbose: bool,

    // User configurations
    frequency: Option<String>,
    generators: Option<Vec<String>>,
}

#[derive(Default, Deserialize)]
struct UserConfig {
    frequency: Option<String>,
    generators: Option<Vec<String>>,
}

impl Config {
    pub fn new(verbose: bool) -> Self {
        let UserConfig {
            frequency,
            generators,
        } = Config::read_config_file_if_exists();
        Self {
            frequency,
            generators,
            verbose,
        }
    }

    pub fn print_if_verbose(&self, message: &str) {
        if self.verbose {
            println!("{}", message);
        }
    }

    fn read_config_file_if_exists() -> UserConfig {
        let data = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .map(|dirs| dirs.data_dir().join("config.json"))
            .filter(|path| path.exists())
            .and_then(|path| fs::read_to_string(path).ok());
        if let Some(data) = data {
            // TODO: add proper error handling and throwing here
            serde_json::from_str(&data)
                .expect("Config should be correct, TODO: add proper error here")
        } else {
            UserConfig::default()
        }
    }
}

// TODO - v1.1.0: add testing here
