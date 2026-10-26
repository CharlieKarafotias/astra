use super::{
    frequency::Frequency,
    generators::{Generators, JuliaConfig, NasaApodConfig, SolidConfig, SpotlightConfig},
    theme::ThemeConfigs,
};
use schemars::generate::SchemaSettings;
use schemars::{JsonSchema, SchemaGenerator};
use serde::Deserialize;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

/// Astra's optional JSON configuration.
#[derive(Debug, Default, Deserialize, JsonSchema, PartialEq)]
#[schemars(title = "Astra Configuration")]
pub(super) struct UserConfig {
    /// Remove cached wallpapers older than this frequency (for example `"1w"`).
    pub(super) auto_clean: Option<Frequency>,
    /// How often Astra should update the wallpaper automatically (for example `"1d"`).
    pub(super) frequency: Option<Frequency>,
    /// Generators Astra may choose from. One is picked at random on each run.
    pub(super) generators: Option<Generators>,
    /// Julia fractal generator options.
    pub(super) julia_gen: Option<JuliaConfig>,
    /// NASA Astronomy Picture of the Day generator options.
    pub(super) nasa_apod_gen: Option<NasaApodConfig>,
    /// Solid-color generator options.
    pub(super) solid_gen: Option<SolidConfig>,
    /// Bing Spotlight generator options.
    pub(super) spotlight_gen: Option<SpotlightConfig>,
    /// Custom color themes used by generators that respect them.
    pub(super) themes: Option<ThemeConfigs>,
    // IF New user config fields, ensure you push_field! in Display impl below & update readme
}

impl UserConfig {
    /// Renders the configuration as a draft-07 JSON Schema for editors and
    /// tooling. See `docs/config.schema.json` for the committed copy.
    pub(super) fn json_schema() -> String {
        let schema =
            SchemaGenerator::new(SchemaSettings::draft07()).into_root_schema_for::<UserConfig>();
        serde_json::to_string_pretty(&schema).expect("configuration schema is serializable")
    }
}

impl Display for UserConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut fields = vec![];

        macro_rules! push_field {
            ($field:ident) => {
                if let Some(val) = &self.$field {
                    fields.push(format!("{}: {}", stringify!($field), val));
                }
            };
        }

        push_field!(auto_clean);
        push_field!(frequency);
        push_field!(generators);
        push_field!(julia_gen);
        push_field!(nasa_apod_gen);
        push_field!(solid_gen);
        push_field!(spotlight_gen);
        push_field!(themes);

        for (index, field) in fields.iter().enumerate() {
            if index == fields.len() - 1 {
                write!(f, "  {}", field)?;
            } else {
                writeln!(f, "  {}", field)?;
            }
        }
        Ok(())
    }
}
