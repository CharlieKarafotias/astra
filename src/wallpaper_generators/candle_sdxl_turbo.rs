//! Local text-to-image generation with Candle and SDXL Turbo.
//!
//! The model code (UNet, VAE, CLIP, scheduler) is Candle's, and this pipeline is
//! adapted from `candle-examples/examples/stable-diffusion` in the
//! <https://github.com/huggingface/candle> repository (MIT/Apache-2.0). Only the
//! wallpaper-specific plumbing (validation, device tiering, output, logging) is
//! Astra's.
//!
//! No Python, no subprocess and no URLs are involved here: every weight path comes
//! from [`super::download`] and points at a local file in the Hugging Face cache.

use anyhow::{Context, Result, anyhow, bail};
use candle_core::{D, DType, Device, IndexOp, Module, Tensor};
use candle_transformers::models::stable_diffusion;
use std::env;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokenizers::Tokenizer;

use super::device;
use super::download::ensure_model_files;
use super::prompt;
use super::utils::{AstraImage, WallpaperGeneratorError};
use crate::cli::SdxlOptions;
use crate::configuration::Config;
use crate::os_implementations::get_screen_resolution;

/// SDXL Turbo's VAE scaling factor (not the 0.18215 used by SDXL base).
const VAE_SCALE: f64 = 0.13025;
/// The pipeline area used when fitting the screen resolution to SDXL.
const TARGET_PIXELS: f64 = 1344.0 * 768.0;
/// Area used by `--fast`. Roughly half the default pixel count.
const TARGET_PIXELS_FAST: f64 = 0.6 * 1024.0 * 1024.0;
/// SDXL gets unreliable above this; also a hard memory-safety limit.
const MAX_DIMENSION: usize = 2048;
const DEFAULT_STEPS: usize = 4;
/// `--fast` trades a little detail for a much shorter run.
const FAST_STEPS: usize = 3;
const MAX_STEPS: usize = 10;

#[derive(Clone, Debug, PartialEq)]
pub struct GenerateRequest {
    pub prompt: String,
    pub negative_prompt: Option<String>,
    pub width: usize,
    pub height: usize,
    pub steps: usize,
    pub guidance_scale: f64,
    pub seed: u64,
    pub allow_cpu: bool,
}

#[derive(Debug)]
pub struct GenerateOutput {
    pub image: AstraImage,
    pub seed: u64,
    pub device: String,
    pub dtype: DType,
    pub elapsed: Duration,
}

/// Validate everything that can be checked before touching the network or a device.
pub fn validate(request: &GenerateRequest) -> Result<()> {
    if request.prompt.trim().is_empty() {
        bail!(
            "the prompt is empty. Pass --prompt \"...\" or set `sdxl_gen.prompt` / \
             `sdxl_gen.characteristics` in your config"
        );
    }
    if request.width == 0 || request.height == 0 {
        bail!("width and height must be greater than zero");
    }
    if !request.width.is_multiple_of(8) || !request.height.is_multiple_of(8) {
        bail!(
            "width and height must be multiples of 8 (requested {}x{}). \
             Valid examples: 1024x1024, 1344x768, 1920x1080",
            request.width,
            request.height
        );
    }
    if request.width > MAX_DIMENSION || request.height > MAX_DIMENSION {
        bail!(
            "width and height must be at most {} pixels (requested {}x{}). \
             Valid examples: 1024x1024, 1344x768, 1920x1080",
            MAX_DIMENSION,
            request.width,
            request.height
        );
    }
    if request.steps == 0 || request.steps > MAX_STEPS {
        bail!(
            "steps must be between 1 and {MAX_STEPS} (SDXL Turbo needs 1-4, got {})",
            request.steps
        );
    }
    if !request.guidance_scale.is_finite() || request.guidance_scale < 0.0 {
        bail!(
            "guidance scale must be a non-negative number (got {})",
            request.guidance_scale
        );
    }
    Ok(())
}

/// Fit an arbitrary screen size to SDXL: preserve the aspect ratio, keep the
/// canvas around `target_pixels`, and round both sides to multiples of 8 (the
/// only size constraint SDXL itself imposes).
pub fn fit_to_sdxl_with_target(width: usize, height: usize, target_pixels: f64) -> (usize, usize) {
    let width = width.max(1);
    let height = height.max(1);
    let scale = (target_pixels / (width as f64 * height as f64)).sqrt();
    let scaled_width = width as f64 * scale;
    let scaled_height = height as f64 * scale;

    let fitted_width = (((scaled_width / 8.0).round() as usize) * 8).clamp(8, MAX_DIMENSION);
    let fitted_height = (((scaled_height / 8.0).round() as usize) * 8).clamp(8, MAX_DIMENSION);
    (fitted_width, fitted_height)
}

/// Fit a screen to the default (quality) canvas.
pub fn fit_to_sdxl(width: usize, height: usize) -> (usize, usize) {
    fit_to_sdxl_with_target(width, height, TARGET_PIXELS)
}

/// The screen resolution fitted to SDXL. Used when `--width/--height` are omitted.
pub fn screen_dimensions(fast: bool) -> Result<(usize, usize)> {
    let (width, height) = get_screen_resolution()
        .map_err(|e| anyhow!("could not detect the screen resolution: {e}"))?;
    let (width, height) = (width as usize, height as usize);
    Ok(if fast {
        fit_to_sdxl_with_target(width, height, TARGET_PIXELS_FAST)
    } else {
        fit_to_sdxl(width, height)
    })
}

fn text_embeddings(
    prompt: &str,
    unconditional_prompt: &str,
    tokenizer_path: &Path,
    clip_weights: &Path,
    clip_config: &stable_diffusion::clip::Config,
    device: &Device,
    use_guidance: bool,
) -> Result<Tensor> {
    let tokenizer = Tokenizer::from_file(tokenizer_path).map_err(|e| {
        anyhow!(
            "could not load tokenizer {}: {e}. Run `astra models download`",
            tokenizer_path.display()
        )
    })?;

    let pad_id = match &clip_config.pad_with {
        Some(padding) => *tokenizer
            .get_vocab(true)
            .get(padding.as_str())
            .ok_or_else(|| anyhow!("tokenizer has no padding token {padding:?}"))?,
        None => *tokenizer
            .get_vocab(true)
            .get("<|endoftext|>")
            .ok_or_else(|| anyhow!("tokenizer has no <|endoftext|> token"))?,
    };

    let encode = |text: &str| -> Result<Tensor> {
        let mut tokens = tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("could not tokenize the prompt: {e}"))?
            .get_ids()
            .to_vec();
        // SDXL CLIP is limited to 77 positions; truncate rather than fail.
        tokens.truncate(clip_config.max_position_embeddings);
        while tokens.len() < clip_config.max_position_embeddings {
            tokens.push(pad_id);
        }
        Ok(Tensor::new(tokens.as_slice(), device)?.unsqueeze(0)?)
    };

    let text_model =
        stable_diffusion::build_clip_transformer(clip_config, clip_weights, device, DType::F32)
            .with_context(|| {
                format!(
                    "could not load the CLIP text encoder {}. Run `astra models download`",
                    clip_weights.display()
                )
            })?;

    let embeddings = text_model.forward(&encode(prompt)?)?;
    if use_guidance {
        let unconditional = text_model.forward(&encode(unconditional_prompt)?)?;
        Ok(Tensor::cat(&[unconditional, embeddings], 0)?)
    } else {
        Ok(embeddings)
    }
}

/// Run the full pipeline and return the decoded image. All inference is local.
pub fn generate(config: &Config, request: &GenerateRequest) -> Result<GenerateOutput> {
    validate(request)?;

    let started = Instant::now();
    let selected = device::require_generation_device(request.allow_cpu)?;
    println!(
        "Device: {} | model: stabilityai/sdxl-turbo | size: {}x{} | steps: {} | guidance: {} | seed: {}",
        selected.description(),
        request.width,
        request.height,
        request.steps,
        request.guidance_scale,
        request.seed
    );

    selected
        .device
        .set_seed(request.seed)
        .context("could not set the random seed on the selected device")?;

    // Attention slicing is deliberately *not* enabled. In SDXL the only
    // high-resolution attention stage runs after the first downsample, so at
    // 1344x768 the largest attention map is 84x48 tokens (~1.3 GiB peak in
    // f32), which fits comfortably. Candle 0.11's sliced-attention path is also
    // broken (it stacks the slices and then treats the result as rank 3), so
    // `Some(..)` here would panic inside the UNet.
    let sd_config = stable_diffusion::StableDiffusionConfig::sdxl_turbo(
        None,
        Some(request.height),
        Some(request.width),
    );
    let mut scheduler = sd_config.build_scheduler(request.steps)?;

    let files = ensure_model_files(config)?;
    let use_guidance = request.guidance_scale > 1.0;

    let phase = Instant::now();
    println!("Encoding the prompt (two CLIP encoders)...");
    // The text encoders run on the CPU and are dropped before the UNet loads.
    // CLIP-G is ~2.8 GB in f32; keeping it on the GPU of a 16 GiB Mac would
    // count against Metal's working set for no speed benefit (it is one pass).
    let text_device = Device::Cpu;
    let unconditional_prompt = request.negative_prompt.clone().unwrap_or_default();
    let clip_embeddings = text_embeddings(
        &request.prompt,
        &unconditional_prompt,
        &files.tokenizer,
        &files.clip,
        &sd_config.clip,
        &text_device,
        use_guidance,
    )?;
    let clip2_embeddings = text_embeddings(
        &request.prompt,
        &unconditional_prompt,
        &files.tokenizer2,
        &files.clip2,
        sd_config
            .clip2
            .as_ref()
            .ok_or_else(|| anyhow!("the SDXL Turbo config is missing its second CLIP encoder"))?,
        &text_device,
        use_guidance,
    )?;
    let text_embeddings = Tensor::cat(&[clip_embeddings, clip2_embeddings], D::Minus1)?
        .to_device(&selected.device)?
        .to_dtype(selected.dtype)?;
    println!("  text encoding took {:.1}s", phase.elapsed().as_secs_f32());

    let generation_context = || {
        format!(
            "generation failed on {} with {} at {}x{}. If this is out of memory, try a \
             smaller --width/--height (for example 1024x1024){}",
            selected.kind,
            format_args!("{:?}", selected.dtype),
            request.width,
            request.height,
            if selected.kind == device::DeviceKind::Cpu {
                ""
            } else {
                " or rerun with --allow-cpu"
            }
        )
    };

    // Candle's Metal convolution path materialises im2col buffers that are far
    // larger than the VAE weights; at 1344x768 the decode alone exceeds Metal's
    // working set on a 16 GiB Mac. Decoding in f32 on the CPU is a few seconds
    // slower but bounded, while CUDA keeps using the GPU.
    let decode_on_cpu = selected.kind == device::DeviceKind::Metal;
    let vae_device = if decode_on_cpu {
        Device::Cpu
    } else {
        selected.device.clone()
    };
    let vae_dtype = if decode_on_cpu {
        DType::F32
    } else {
        selected.dtype
    };

    println!("Loading the UNet...");
    let phase = Instant::now();
    let unet = sd_config
        .build_unet(&files.unet, &selected.device, 4, false, selected.dtype)
        .with_context(|| {
            format!(
                "could not load the UNet {}. Run `astra models download`",
                files.unet.display()
            )
        })
        .with_context(generation_context)?;
    println!("  UNet loaded in {:.1}s", phase.elapsed().as_secs_f32());

    let latents = Tensor::randn(
        0f32,
        1f32,
        (1, 4, request.height / 8, request.width / 8),
        &selected.device,
    )?;
    let mut latents = (latents * scheduler.init_noise_sigma())?.to_dtype(selected.dtype)?;
    let timesteps = scheduler.timesteps().to_vec();

    println!("Starting sampling ({} steps)...", timesteps.len());
    for (index, &timestep) in timesteps.iter().enumerate() {
        let step_started = Instant::now();
        let latent_input = if use_guidance {
            Tensor::cat(&[&latents, &latents], 0)?
        } else {
            latents.clone()
        };
        let latent_input = scheduler.scale_model_input(latent_input, timestep)?;
        let noise_pred = unet
            .forward(&latent_input, timestep as f64, &text_embeddings)
            .with_context(generation_context)?;

        let noise_pred = if use_guidance {
            let noise_pred = noise_pred.chunk(2, 0)?;
            let unconditional = &noise_pred[0];
            let text = &noise_pred[1];
            (unconditional + ((text - unconditional)? * request.guidance_scale)?)?
        } else {
            noise_pred
        };

        latents = scheduler
            .step(&noise_pred, timestep, &latents)
            .with_context(generation_context)?;
        println!(
            "step {}/{} done in {:.2}s",
            index + 1,
            timesteps.len(),
            step_started.elapsed().as_secs_f32()
        );
    }

    // The UNet is by far the largest allocation (~5 GiB in f16). Free it before
    // the decoder runs; on Metal the decoder reads the latents on the CPU, so
    // make sure the sampling command buffers have completed first.
    println!("Decoding latents...");
    selected.device.synchronize()?;
    drop(unet);
    println!("Loading the VAE...");
    let phase = Instant::now();
    let vae = sd_config
        .build_vae(&files.vae, &vae_device, vae_dtype)
        .with_context(|| {
            format!(
                "could not load the VAE {}. Run `astra models download`",
                files.vae.display()
            )
        })?;
    println!("  VAE loaded in {:.1}s", phase.elapsed().as_secs_f32());
    let phase = Instant::now();
    let latents = latents.to_device(&vae_device)?.to_dtype(vae_dtype)?;
    let image = vae
        .decode(&(&latents / VAE_SCALE)?)
        .with_context(generation_context)?;
    println!("  VAE decode took {:.1}s", phase.elapsed().as_secs_f32());
    let image = ((image / 2.0)? + 0.5)?.to_device(&Device::Cpu)?;
    let image = image.clamp(0f32, 1.0)?;
    let image = (image * 255.0)?.to_dtype(DType::U8)?.i(0)?;
    let (channels, height, width) = image.dims3()?;
    if channels != 3 {
        bail!("the VAE produced an image with {channels} channels, expected 3");
    }
    let pixels = image.permute((1, 2, 0))?.flatten_all()?.to_vec1::<u8>()?;
    let image = AstraImage::from_raw(width as u32, height as u32, pixels)
        .ok_or_else(|| anyhow!("the VAE produced an invalid {width}x{height} RGB buffer"))?;

    Ok(GenerateOutput {
        image,
        seed: request.seed,
        device: selected.description(),
        dtype: selected.dtype,
        elapsed: started.elapsed(),
    })
}

/// Resolve a prompt, seed, size and steps from CLI options plus `sdxl_gen` config.
fn resolve_request(config: &Config, options: &SdxlOptions) -> Result<GenerateRequest> {
    let seed = options
        .seed
        .or_else(|| config.sdxl_gen().and_then(|sdxl| sdxl.seed()))
        .unwrap_or_else(prompt::random_seed);

    let configured_prompt = config.sdxl_gen().and_then(|sdxl| sdxl.prompt());
    let prompt_text = match options.prompt.clone().or(configured_prompt) {
        Some(prompt) => prompt,
        None => {
            let characteristics = prompt::characteristics(config);
            prompt::generate_prompt(&characteristics, seed)
        }
    };

    let (width, height) = match (options.width, options.height) {
        (Some(width), Some(height)) => (width as usize, height as usize),
        (None, None) => screen_dimensions(options.fast)?,
        _ => {
            bail!("pass both --width and --height, or neither to use the fitted screen resolution")
        }
    };

    let default_steps = if options.fast {
        FAST_STEPS
    } else {
        DEFAULT_STEPS
    };

    Ok(GenerateRequest {
        prompt: prompt_text,
        negative_prompt: options
            .negative_prompt
            .clone()
            .or_else(|| config.sdxl_gen().and_then(|sdxl| sdxl.negative_prompt())),
        width,
        height,
        steps: options
            .steps
            .or_else(|| config.sdxl_gen().and_then(|sdxl| sdxl.steps()))
            .unwrap_or(default_steps as u32) as usize,
        guidance_scale: options
            .guidance_scale
            .or_else(|| config.sdxl_gen().and_then(|sdxl| sdxl.guidance_scale()))
            .unwrap_or(0.0),
        seed,
        allow_cpu: options.allow_cpu,
    })
}

/// Where an extra copy of the image should be written, if at all.
fn extra_output_path(options: &SdxlOptions) -> Option<PathBuf> {
    match &options.output {
        Some(path) if path.is_absolute() => Some(path.clone()),
        Some(path) => match env::var_os("ASTRA_OUTPUT_DIR") {
            Some(dir) => Some(PathBuf::from(dir).join(path)),
            None => Some(path.clone()),
        },
        None => env::var_os("ASTRA_OUTPUT_DIR").map(|dir| {
            PathBuf::from(dir).join(format!(
                "wallpaper-{}.png",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs()
            ))
        }),
    }
}

/// CLI entry point: generate, print a summary, and optionally write `--output`.
pub fn generate_sdxl(
    config: &Config,
    options: &SdxlOptions,
) -> std::result::Result<AstraImage, WallpaperGeneratorError> {
    let request = resolve_request(config, options)
        .map_err(|e| WallpaperGeneratorError::ImageGeneration(format!("{e:#}")))?;

    if request.guidance_scale <= 1.0 && request.negative_prompt.is_some() {
        config.print_if_verbose(
            "NOTE: SDXL Turbo runs at guidance <= 1.0, so the negative prompt is ignored. \
             Set --guidance-scale above 1.0 to use it (slower, and not what Turbo is tuned for).",
        );
    }

    let output = generate(config, &request)
        .map_err(|e| WallpaperGeneratorError::ImageGeneration(format!("{e:#}")))?;

    if let Some(path) = extra_output_path(options) {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent).map_err(|e| {
                WallpaperGeneratorError::ImageGeneration(format!(
                    "could not create output directory {}: {e}",
                    parent.display()
                ))
            })?;
        }
        output.image.save(&path).map_err(|e| {
            WallpaperGeneratorError::ImageGeneration(format!(
                "could not save the image to {}: {e}",
                path.display()
            ))
        })?;
        println!("Wrote {}", path.display());
    }

    println!(
        "Prompt: {}\nDevice: {}\nModel: stabilityai/sdxl-turbo\nDtype: {:?}\nSize: {}x{}\nSteps: {}\nSeed: {}\nElapsed: {:.1}s",
        request.prompt,
        output.device,
        output.dtype,
        request.width,
        request.height,
        request.steps,
        output.seed,
        output.elapsed.as_secs_f32()
    );

    Ok(output.image)
}

/// Validate everything without downloading or running diffusion.
pub fn dry_run_sdxl(
    config: &Config,
    options: &SdxlOptions,
) -> std::result::Result<(), WallpaperGeneratorError> {
    let request = resolve_request(config, options)
        .map_err(|e| WallpaperGeneratorError::ImageGeneration(format!("{e:#}")))?;
    validate(&request).map_err(|e| WallpaperGeneratorError::ImageGeneration(format!("{e:#}")))?;

    let detected = device::detect_device();
    let tier = device::pick_tier(detected.kind, detected.total_memory_gb);
    println!("Dry run - nothing will be downloaded or generated.");
    if options.fast {
        println!("Preset: fast (~0.6 MP, 3 steps unless overridden)");
    }
    println!("Prompt: {}", request.prompt);
    if let Some(negative) = &request.negative_prompt {
        println!(
            "Negative prompt: {negative}{}",
            if request.guidance_scale <= 1.0 {
                " (ignored at guidance <= 1.0)"
            } else {
                ""
            }
        );
    }
    println!(
        "Size: {}x{} | steps: {} | guidance: {} | seed: {}",
        request.width, request.height, request.steps, request.guidance_scale, request.seed
    );
    println!(
        "Device: {} (tier: {})",
        detected.description(),
        match tier {
            device::Tier::GpuTurbo => "GPU + Turbo",
            device::Tier::CpuFallback => "CPU fallback",
        }
    );
    if tier == device::Tier::CpuFallback && !request.allow_cpu {
        println!(
            "Real generation would require --allow-cpu on this machine. \
             Build with --features metal (macOS) or --features cuda,cudnn (Nvidia) for the GPU path."
        );
    }
    println!("Model cache: {}", super::download::cache_dir().display());
    Ok(())
}

/// Whether auto mode can select SDXL on this machine.
pub fn sdxl_can_run_on_this_machine() -> bool {
    device::can_run_sdxl()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> GenerateRequest {
        GenerateRequest {
            prompt: "a mountain lake".to_string(),
            negative_prompt: None,
            width: 1344,
            height: 768,
            steps: 4,
            guidance_scale: 0.0,
            seed: 42,
            allow_cpu: false,
        }
    }

    #[test]
    fn test_validate_accepts_a_good_request() {
        assert!(validate(&request()).is_ok());
    }

    #[test]
    fn test_validate_rejects_empty_prompt() {
        let mut request = request();
        request.prompt = "   ".to_string();
        assert!(
            validate(&request)
                .unwrap_err()
                .to_string()
                .contains("empty")
        );
    }

    #[test]
    fn test_validate_rejects_non_multiple_of_eight() {
        let mut request = request();
        request.width = 1001;
        assert!(
            validate(&request)
                .unwrap_err()
                .to_string()
                .contains("multiples of 8")
        );
    }

    #[test]
    fn test_validate_rejects_oversized_dimensions() {
        let mut request = request();
        request.width = 4096;
        assert!(
            validate(&request)
                .unwrap_err()
                .to_string()
                .contains("at most")
        );
    }

    #[test]
    fn test_validate_rejects_bad_steps() {
        let mut request = request();
        request.steps = 0;
        assert!(validate(&request).is_err());
        request.steps = 11;
        assert!(validate(&request).is_err());
    }

    #[test]
    fn test_fit_to_sdxl_preserves_aspect_and_bounds() {
        let (width, height) = fit_to_sdxl(2560, 1600);
        assert_eq!(width % 8, 0);
        assert_eq!(height % 8, 0);
        assert!((width as f64 / height as f64 - 1.6).abs() < 0.03);
        assert!(width <= MAX_DIMENSION && height <= MAX_DIMENSION);

        let (width, height) = fit_to_sdxl(1920, 1080);
        assert!(width <= MAX_DIMENSION && height <= MAX_DIMENSION);
        assert!((width as f64 / height as f64 - 16.0 / 9.0).abs() < 0.03);

        // Very large screens are scaled down to the ~1 MP target, not up to 2048.
        let (width, height) = fit_to_sdxl(10_000, 10_000);
        assert!(width <= MAX_DIMENSION && height <= MAX_DIMENSION);
        assert!((width as f64 * height as f64 - TARGET_PIXELS).abs() < TARGET_PIXELS * 0.1);
    }

    #[test]
    fn test_fast_preset_is_about_half_the_pixels_and_keeps_aspect() {
        let (fast_width, fast_height) = fit_to_sdxl_with_target(2560, 1600, TARGET_PIXELS_FAST);
        assert_eq!(fast_width % 8, 0);
        assert_eq!(fast_height % 8, 0);
        assert!((fast_width as f64 / fast_height as f64 - 1.6).abs() < 0.03);

        let fast_pixels = (fast_width * fast_height) as f64;
        assert!(
            (fast_pixels - TARGET_PIXELS_FAST).abs() < TARGET_PIXELS_FAST * 0.1,
            "fast canvas {fast_width}x{fast_height} is not near {TARGET_PIXELS_FAST} pixels"
        );

        let (full_width, full_height) = fit_to_sdxl(2560, 1600);
        assert!(fast_width * fast_height < full_width * full_height);
    }

    // The pure-Rust assertion lives in a separate integration test so the
    // scanned sources do not contain the needles.
}
