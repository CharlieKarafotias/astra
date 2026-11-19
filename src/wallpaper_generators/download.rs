//! Model file resolution and caching through the Hugging Face Hub.
//!
//! Every file goes through `hf-hub`'s standard cache (the same one used by the
//! `hf` CLI), so a model pre-warmed with `astra models download` is equally
//! available to `hf download` and vice versa. Paths returned by this module are
//! always local filesystem paths; Candle never sees a Hub URL.

use anyhow::{Context, Result};
use hf_hub::Cache;
use hf_hub::api::sync::{Api, ApiBuilder};
use std::fs;
use std::path::PathBuf;

use crate::configuration::Config;

pub const SDXL_TURBO_REPO: &str = "stabilityai/sdxl-turbo";
pub const VAE_REPO: &str = "madebyollin/sdxl-vae-fp16-fix";
pub const TOKENIZER_REPO: &str = "openai/clip-vit-large-patch14";
pub const TOKENIZER2_REPO: &str = "laion/CLIP-ViT-bigG-14-laion2B-39B-b160k";

/// One file needed by the pipeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelFileSpec {
    pub repo: &'static str,
    pub file: &'static str,
    pub label: &'static str,
}

/// The files needed for SDXL Turbo.
///
/// The fp16 variants are used for every device: on a GPU they load as F16, and on
/// the CPU `VarBuilder` converts them to F32. That halves the download size
/// without changing the result. The VAE comes from `sdxl-vae-fp16-fix` because the
/// original SDXL VAE overflows in fp16 (huggingface/candle#1060).
pub const MODEL_FILES: [ModelFileSpec; 6] = [
    ModelFileSpec {
        repo: TOKENIZER_REPO,
        file: "tokenizer.json",
        label: "CLIP tokenizer",
    },
    ModelFileSpec {
        repo: TOKENIZER2_REPO,
        file: "tokenizer.json",
        label: "CLIP-G tokenizer",
    },
    ModelFileSpec {
        repo: SDXL_TURBO_REPO,
        file: "text_encoder/model.fp16.safetensors",
        label: "CLIP text encoder",
    },
    ModelFileSpec {
        repo: SDXL_TURBO_REPO,
        file: "text_encoder_2/model.fp16.safetensors",
        label: "CLIP-G text encoder",
    },
    ModelFileSpec {
        repo: SDXL_TURBO_REPO,
        file: "unet/diffusion_pytorch_model.fp16.safetensors",
        label: "SDXL Turbo UNet",
    },
    ModelFileSpec {
        repo: VAE_REPO,
        file: "diffusion_pytorch_model.safetensors",
        label: "SDXL fp16-safe VAE",
    },
];

/// Local paths passed to Candle.
#[derive(Clone, Debug)]
pub struct ModelFiles {
    pub tokenizer: PathBuf,
    pub tokenizer2: PathBuf,
    pub clip: PathBuf,
    pub clip2: PathBuf,
    pub unet: PathBuf,
    pub vae: PathBuf,
}

/// The Hugging Face cache root (`$HF_HOME/hub`, or `~/.cache/huggingface/hub`).
pub fn cache_dir() -> PathBuf {
    Cache::from_env().path().clone()
}

fn api() -> Result<Api> {
    let mut builder = ApiBuilder::from_env();
    // hf-hub 0.4 only reads the token file on its own. Respect `HF_TOKEN` too,
    // which is the override documented by Astra. Never print the value.
    if let Ok(token) = std::env::var("HF_TOKEN") {
        let token = token.trim();
        if !token.is_empty() {
            builder = builder.with_token(Some(token.to_string()));
        }
    }
    builder
        .build()
        .context("could not initialise the Hugging Face Hub client (check HF_HOME/HF_TOKEN)")
}

/// Return the local path of a file if it is already in the Hub cache.
pub fn cached_path(spec: &ModelFileSpec) -> Option<PathBuf> {
    Cache::from_env()
        .model(spec.repo.to_string())
        .get(spec.file)
}

/// Resolve every model file, downloading anything missing from the Hub.
pub fn ensure_model_files(config: &Config) -> Result<ModelFiles> {
    let api = api()?;
    let mut paths = Vec::with_capacity(MODEL_FILES.len());

    for spec in MODEL_FILES.iter() {
        if let Some(path) = cached_path(spec) {
            config.print_if_verbose(&format!("Using cached {}: {}", spec.label, path.display()));
            paths.push(path);
            continue;
        }

        println!(
            "Downloading {} ({}/{})...",
            spec.label, spec.repo, spec.file
        );
        let path = api
            .model(spec.repo.to_string())
            .get(spec.file)
            .with_context(|| {
                format!(
                    "failed to download {} from {}/{}. Check your network connection and \
                     HF_TOKEN, then retry `astra models download`",
                    spec.label, spec.repo, spec.file
                )
            })?;
        config.print_if_verbose(&format!("Cached {} at {}", spec.label, path.display()));
        paths.push(path);
    }

    Ok(ModelFiles {
        tokenizer: paths[0].clone(),
        tokenizer2: paths[1].clone(),
        clip: paths[2].clone(),
        clip2: paths[3].clone(),
        unet: paths[4].clone(),
        vae: paths[5].clone(),
    })
}

/// `astra models download`: fetch every file without generating anything.
pub fn download_all(config: &Config) -> Result<()> {
    let files = ensure_model_files(config)?;
    println!(
        "All SDXL Turbo files are cached in {}",
        cache_dir().display()
    );
    for path in [
        &files.tokenizer,
        &files.tokenizer2,
        &files.clip,
        &files.clip2,
        &files.unet,
        &files.vae,
    ] {
        println!("  {}", path.display());
    }
    Ok(())
}

/// `astra models list`: show what is cached. Never touches the network.
pub fn print_model_list() {
    println!("Hugging Face cache: {}", cache_dir().display());
    println!();
    for status in list_model_files() {
        match (&status.path, status.size_bytes) {
            (Some(path), Some(size)) => println!(
                "cached   {:>7.1} MB  {:<22} {}",
                size as f64 / (1024.0 * 1024.0),
                status.spec.label,
                path.display()
            ),
            (Some(path), None) => {
                println!(
                    "cached   {:>7}      {:<22} {}",
                    "?",
                    status.spec.label,
                    path.display()
                )
            }
            (None, _) => println!(
                "missing  {:>7}      {:<22} {}/{}",
                "-", status.spec.label, status.spec.repo, status.spec.file
            ),
        }
    }
    println!();
    println!("Run `astra models download` to fetch anything missing.");
}

/// Cache status of one model file.
#[derive(Clone, Debug)]
pub struct ModelFileStatus {
    pub spec: ModelFileSpec,
    pub path: Option<PathBuf>,
    pub size_bytes: Option<u64>,
}

/// Inspect the cache without downloading anything.
pub fn list_model_files() -> Vec<ModelFileStatus> {
    MODEL_FILES
        .iter()
        .map(|spec| {
            let path = cached_path(spec);
            let size_bytes = path
                .as_ref()
                .and_then(|path| fs::metadata(path).ok())
                .map(|metadata| metadata.len());
            ModelFileStatus {
                spec: *spec,
                path,
                size_bytes,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_files_cover_both_encoders_and_the_fixed_vae() {
        assert_eq!(MODEL_FILES.len(), 6);
        assert!(MODEL_FILES.iter().any(|f| f.repo == VAE_REPO));
        assert!(
            MODEL_FILES
                .iter()
                .any(|f| f.file.contains("text_encoder_2"))
        );
    }

    #[test]
    fn test_cache_dir_ends_with_hub() {
        let dir = cache_dir();
        assert_eq!(dir.file_name().and_then(|n| n.to_str()), Some("hub"));
    }

    #[test]
    fn test_cached_path_is_none_for_unlikely_file() {
        let spec = ModelFileSpec {
            repo: "astra/this-repo-does-not-exist",
            file: "nope.safetensors",
            label: "test",
        };
        assert!(cached_path(&spec).is_none());
    }
}
