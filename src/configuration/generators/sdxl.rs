use schemars::JsonSchema;
use serde::Deserialize;
use std::fmt::{Display, Formatter, Write};

/// Options for the local SDXL Turbo (pure Rust) generator.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq)]
pub struct SdxlConfig {
    /// Fixed prompt. When unset, a prompt is generated from `characteristics`.
    prompt: Option<String>,
    /// Negative prompt. SDXL Turbo runs at guidance 0, where it has no effect.
    negative_prompt: Option<String>,
    /// Traits the generated prompt should include (for example `"golden hour"`).
    /// When empty or missing, a default set of wallpaper characteristics is used.
    characteristics: Option<Vec<String>>,
    /// Number of denoising steps (1-10; SDXL Turbo needs 1-4).
    steps: Option<u32>,
    /// Classifier-free guidance scale. SDXL Turbo is distilled for 0.0.
    guidance_scale: Option<f64>,
    /// Fixed random seed. The same prompt, size, steps and seed reproduce an image.
    seed: Option<u64>,
}

impl SdxlConfig {
    pub fn prompt(&self) -> Option<String> {
        self.prompt.clone()
    }

    pub fn negative_prompt(&self) -> Option<String> {
        self.negative_prompt.clone()
    }

    pub fn characteristics(&self) -> Option<Vec<String>> {
        self.characteristics.clone()
    }

    pub fn steps(&self) -> Option<u32> {
        self.steps
    }

    pub fn guidance_scale(&self) -> Option<f64> {
        self.guidance_scale
    }

    pub fn seed(&self) -> Option<u64> {
        self.seed
    }
}

impl Display for SdxlConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        if let Some(val) = &self.prompt {
            writeln!(&mut s, "    prompt: {}", val)?;
        }
        if let Some(val) = &self.negative_prompt {
            writeln!(&mut s, "    negative_prompt: {}", val)?;
        }
        if let Some(val) = &self.characteristics {
            writeln!(&mut s, "    characteristics: {}", val.join(", "))?;
        }
        if let Some(val) = &self.steps {
            writeln!(&mut s, "    steps: {}", val)?;
        }
        if let Some(val) = &self.guidance_scale {
            writeln!(&mut s, "    guidance_scale: {}", val)?;
        }
        if let Some(val) = &self.seed {
            writeln!(&mut s, "    seed: {}", val)?;
        }
        if !s.is_empty() {
            writeln!(f)?;
            s.pop(); // remove last newline character
        }
        write!(f, "{}", s)
    }
}
