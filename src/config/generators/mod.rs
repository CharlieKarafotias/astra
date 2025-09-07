use super::super::cli::{Generator, SolidMode};
use serde::Deserialize;
use std::fmt::{Display, Formatter};

pub(crate) mod julia;
mod solid;
mod spotlight;

// Any generator config should be added to ALL_GENERATORS with default values (see Generators below)
pub(super) use julia::JuliaConfig;
pub(super) use solid::SolidConfig;
pub(super) use spotlight::SpotlightConfig;

#[macro_export]
macro_rules! respect_user_config_or_default {
    ($config:expr, $gen_config:ident, $field_getter:ident, $closure:expr) => {
        $config
            .respect_user_config
            .then(|| $config.$gen_config())
            .flatten()
            .and_then(|gen_config| gen_config.$field_getter())
            .map(|value| {
                $config.print_if_verbose(&format!(
                    "Using user config for {}",
                    stringify!($field_getter)
                ));
                Ok(value)
            })
            .unwrap_or_else(|| $closure)
    };
}

#[derive(Debug, PartialEq)]
pub struct Generators(pub(super) Vec<Generator>);

impl Generators {
    pub const ALL_GENERATORS: [Generator; 3] = [
        Generator::Julia,
        Generator::Solid {
            mode: SolidMode::Random,
        },
        Generator::Spotlight,
    ];
}

impl Display for Generators {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|g| g.prefix().to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl std::ops::Deref for Generators {
    type Target = Vec<Generator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for Generators {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw: Option<Vec<String>> = Option::deserialize(deserializer)?;
        match raw {
            Some(list) => {
                let parsed = list
                    .into_iter()
                    .map(|s| s.parse().map_err(serde::de::Error::custom))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Generators(parsed))
            }
            None => Ok(Generators(Vec::new())),
        }
    }
}
