pub(crate) mod julia;
mod solid;

pub(super) use julia::JuliaConfig;
pub(super) use solid::SolidConfig;

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
