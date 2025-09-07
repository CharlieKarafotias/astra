use super::{
    frequency::Frequency,
    generators::{Generators, JuliaConfig, SolidConfig, SpotlightConfig},
};
use serde::Deserialize;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Deserialize, PartialEq)]
pub(super) struct UserConfig {
    pub(super) auto_clean: Option<Frequency>,
    pub(super) frequency: Option<Frequency>,
    pub(super) generators: Option<Generators>,
    pub(super) julia_gen: Option<JuliaConfig>,
    pub(super) solid_gen: Option<SolidConfig>,
    pub(super) spotlight_gen: Option<SpotlightConfig>,
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
        push_field!(frequency);
        push_field!(generators);
        push_field!(julia_gen);
        push_field!(solid_gen);
        push_field!(spotlight_gen);

        write!(f, "{}", fields.join(", "))
    }
}
