//! Deterministic wallpaper prompt generation.
//!
//! This is intentionally model-free: a seeded combinatorial template over curated
//! descriptor pools plus the user's `sdxl_gen.characteristics`. It is instant and
//! works offline, which keeps auto mode and CI predictable. The same seed always
//! produces the same prompt.

use crate::configuration::Config;
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

/// Shipped characteristics, used when `sdxl_gen.characteristics` is empty.
pub const DEFAULT_CHARACTERISTICS: &[&str] = &[
    "photorealistic",
    "high detail",
    "no people",
    "no text",
    "wallpaper",
];

const SUBJECTS: &[&str] = &[
    "a lone lighthouse on a rocky coast",
    "snow-capped mountain peaks reflected in an alpine lake",
    "a winding road through an autumn forest",
    "a futuristic city skyline at dusk",
    "a field of lavender under drifting clouds",
    "a turquoise lagoon framed by limestone cliffs",
    "a quiet desert canyon with layered sandstone",
    "a waterfall hidden in a mossy rainforest",
    "a winding river delta seen from above",
    "a medieval coastal village at the edge of the sea",
    "a glacier meeting a black sand beach",
    "a sea of clouds below a mountain ridge",
];

const DETAILS: &[&str] = &[
    "dramatic clouds",
    "soft morning mist",
    "a clear horizon",
    "long shadows",
    "reflections on calm water",
    "delicate atmospheric haze",
];

const LIGHTING: &[&str] = &[
    "golden hour light",
    "blue hour light",
    "soft dawn light",
    "warm sunset glow",
    "moonlit night",
    "bright midday sun",
];

const STYLES: &[&str] = &[
    "photorealistic photograph",
    "fine-art landscape photography",
    "cinematic wide shot",
    "nature documentary still",
];

/// The characteristics to use: the user's list when non-empty, otherwise defaults.
pub fn characteristics(config: &Config) -> Vec<String> {
    config
        .sdxl_gen()
        .and_then(|sdxl| sdxl.characteristics())
        .filter(|values| values.iter().any(|value| !value.trim().is_empty()))
        .unwrap_or_else(|| {
            DEFAULT_CHARACTERISTICS
                .iter()
                .map(|value| value.to_string())
                .collect()
        })
}

/// Compose a prompt for `seed`. Deterministic for the same seed and inputs.
pub fn generate_prompt(characteristics: &[String], seed: u64) -> String {
    let mut rng = StdRng::seed_from_u64(seed);

    let mut parts = vec![
        SUBJECTS[rng.random_range(0..SUBJECTS.len())].to_string(),
        DETAILS[rng.random_range(0..DETAILS.len())].to_string(),
        LIGHTING[rng.random_range(0..LIGHTING.len())].to_string(),
        STYLES[rng.random_range(0..STYLES.len())].to_string(),
    ];

    for characteristic in characteristics {
        let characteristic = characteristic.trim();
        // Keep prompts readable and bounded: skip blanks and duplicates.
        if characteristic.is_empty()
            || parts
                .iter()
                .any(|part| part.eq_ignore_ascii_case(characteristic))
        {
            continue;
        }
        parts.push(characteristic.to_string());
    }

    parts.join(", ")
}

/// A fresh random seed for a run that did not specify one.
pub fn random_seed() -> u64 {
    rand::random::<u64>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn test_prompt_is_deterministic_for_a_seed() {
        let characteristics = owned(&["golden hour", "no text"]);
        assert_eq!(
            generate_prompt(&characteristics, 42),
            generate_prompt(&characteristics, 42)
        );
    }

    #[test]
    fn test_prompt_changes_with_the_seed() {
        let characteristics = owned(&["golden hour"]);
        let prompts: Vec<String> = (0..16)
            .map(|seed| generate_prompt(&characteristics, seed))
            .collect();
        assert!(
            prompts.iter().any(|prompt| prompt != &prompts[0]),
            "expected seed-dependent prompts, got {prompts:?}"
        );
    }

    #[test]
    fn test_characteristics_are_included_once() {
        let prompt = generate_prompt(&owned(&["aurora", "Aurora", "  ", "no people"]), 7);
        assert!(prompt.contains("aurora"));
        assert!(prompt.contains("no people"));
        assert_eq!(prompt.to_lowercase().matches("aurora").count(), 1);
        assert!(!prompt.contains(", ,"));
    }

    #[test]
    fn test_default_characteristics_exist() {
        assert!(!DEFAULT_CHARACTERISTICS.is_empty());
        let prompt = generate_prompt(&owned(DEFAULT_CHARACTERISTICS), 1);
        for characteristic in DEFAULT_CHARACTERISTICS {
            assert!(prompt.contains(characteristic));
        }
    }
}
