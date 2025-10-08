use super::{
    frequency::Frequency,
    generators::{Generators, JuliaConfig, SolidConfig, SpotlightConfig},
    theme::ThemeConfigs,
};
use serde::Deserialize;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub(super) struct UserConfig {
    pub(super) auto_clean: Option<Frequency>,
    // TODO v1.2.0: add frequency back in to control how often wallpaper changes
    // pub(super) frequency: Option<Frequency>,
    pub(super) generators: Option<Generators>,
    pub(super) julia_gen: Option<JuliaConfig>,
    pub(super) solid_gen: Option<SolidConfig>,
    pub(super) spotlight_gen: Option<SpotlightConfig>,
    pub(super) themes: Option<ThemeConfigs>,
    // IF New user config fields, ensure you push_field! in Display impl below & update readme
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
        // TODO v1.2.0: add frequency back in to control how often wallpaper changes
        // push_field!(frequency);
        push_field!(generators);
        push_field!(julia_gen);
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
