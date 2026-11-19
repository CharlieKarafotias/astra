# Astra — Candle + SDXL Turbo (Pure Rust) — Spec for OpenCode

How to use this file: Paste this whole file into OpenCode, or save it as SPEC_SDXL_TURBO.md in the repo root and run OpenCode with: Implement SPEC_SDXL_TURBO.md in this repo. Work on a new branch, do not push.
OpenCode should explore the repo first, then implement. Do not assume file contents — read them.

## 1. Project context

Repo: local Rust CLI at ~/code/cli/astra, branch main (from terminal prompt).
Binary name: astra.

Existing layout (from VS Code explorer — verify by reading the repo, do not assume):

```
astra/
  .github/
  docs/
  src/
    main.rs
    cli.rs
    constants.rs
    configuration/
    os_implementations/
      linux/ (utils/mod.rs, errors.rs, mod.rs, systemd.rs)
      macos/ (utils/mod.rs, errors.rs, launchctl.rs, mod.rs)
      windows/ (mod.rs)
    themes/
    wallpaper_generators/   <-- new generation code lives here
```

Astra is a local wallpaper tool. It should pick a model based on machine specs and generate wallpapers locally.

There is already a wallpaper_generators folder. Merge with it — do not create a second, parallel generation system.

## 2. Goal

Add pure-Rust, local text-to-image generation to Astra using Hugging Face candle + stabilityai/sdxl-turbo.

Single `cargo run` should download the model once, cache it, and generate a wallpaper PNG locally.
No Python, no subprocess, no diffusers, no sidecar, no network calls except to Hugging Face Hub.

Primary use case: `astra generate` creates a photorealistic wallpaper, default 1344x768, in 1–4 steps.

## 3. Non-goals for this task

- No LoRA, no Lightning LoRA fuse. SDXL Turbo is already distilled for few-step — it replaces the LoRA path.
- No UI, no web server, no scheduler/daemon, no upscaler, no img2img, no inpainting.
- No Python code added. If there is a Python generation script in the repo, do not keep it as the generation path. Either delete it in this task with a note, or leave it untouched and make the Rust path the only path wired to the CLI. Do not have two generation paths.
- No model training, no fine-tuning.
- No committing model weights to git. No committing secrets or tokens.

## 4. Background: what this replaces

Previous Python workflow (for reference only, do not port it):

- StableDiffusionXLPipeline, stabilityai/stable-diffusion-xl-base-1.0, fp16
- ByteDance/SDXL-Lightning 4-step LoRA, fused
- MPS, 1344x768, 4 steps, guidance 0
- Prompt: `Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text`
- Saved `wallpaper-santorini.png`

New path keeps the same prompt, size, steps, and guidance, but uses stabilityai/sdxl-turbo in Candle. Turbo needs no LoRA file.

## 5. Technical decision (do not change without asking)

Use candle, Hugging Face's Rust ML framework.

Reference implementation: `candle-examples/examples/stable-diffusion` in the huggingface/candle repo. Do not reinvent UNet / VAE / CLIP / scheduler code — copy and adapt that example into Astra, with a comment attributing it.

`candle-transformers::models::stable_diffusion::StableDiffusionConfig::sdxl_turbo()` is the config to use.

Scheduler for Turbo is Euler Ancestral, not DDIM. This is handled inside `sdxl_turbo()` — do not override it with DDIM.

Turbo VAE scale is `0.13025`, not base SDXL `0.18215`.

## 6. Dependencies and features

Inspect existing `Cargo.toml` first. Keep existing deps (clap, serde, etc.). Add only what is needed, pin versions, do not use `*`.

Add:

```toml
candle-core = "0.9"
candle-transformers = "0.9"
candle-nn = "0.9"
tokenizers = "0.21"
hf-hub = { version = "0.4", features = ["tokio"] }
image = "0.25"
anyhow = "1"
rand = "0.8"
sysinfo = "0.32" (for memory-based tiering)
```

All candle-* crates must be on the same version. If the lockfile pins 0.8.x, use 0.8 for all candle crates — do not mix 0.8 and 0.9.

Features (add to `[features]`):

```toml
metal = ["candle-core/metal", "candle-transformers/metal"]   # Apple Silicon
cuda = ["candle-core/cuda", "candle-transformers/cuda"]      # Nvidia
cudnn = ["candle-core/cudnn", "candle-transformers/cudnn"]   # Nvidia speedup
accelerate = ["candle-core/accelerate"], mkl = ["candle-core/mkl"]  # CPU speedups
```

Default build must work on CPU so `cargo build` works in CI without GPU. Do not enable metal and cuda at the same time in one build. Runtime picks from what was compiled in.

Document in README:
- macOS: `cargo run --release --features metal`
- Nvidia: `cargo run --release --features cuda,cudnn`

## 7. Model: SDXL Turbo — files to download

Repo: `stabilityai/sdxl-turbo`

Verify against the Candle SDXL Turbo example before coding. Expected files:

- Tokenizer 1: `openai/clip-vit-large-patch14` / `tokenizer.json`
- Tokenizer 2: `laion/CLIP-ViT-bigG-14-laion2B-39B-b160k` / `tokenizer.json`
- Text encoder 1: `stabilityai/sdxl-turbo` / `text_encoder/model.fp16.safetensors` (when fp16)
- Text encoder 2: `stabilityai/sdxl-turbo` / `text_encoder_2/model.fp16.safetensors` (when fp16)
- UNet: `stabilityai/sdxl-turbo` / `unet/diffusion_pytorch_model.fp16.safetensors` (when fp16)
- VAE: Do not use the VAE from sdxl-turbo when fp16. Candle has a known fp16 VAE bug for SDXL. Use the fixed VAE: `madebyollin/sdxl-vae-fp16-fix` / `diffusion_pytorch_model.safetensors`. See https://github.com/huggingface/candle/issues/1060

Scheduler config is code in Candle, not a file.

Defaults:

- width = 1344, height = 768
- steps = 4
- guidance_scale = 0.0
- seed = random if not provided, deterministic if provided
- dtype = F16

Also support 1024x1024 and 1920x1080 via CLI flags. Do not hardcode 1344x768 in the generation function.

## 8. Download and cache

Use hf-hub sync API (`hf_hub::api::sync::Api`).

Respect `HF_HOME` and `HF_TOKEN` env vars. Never hardcode a token, never log a token.

Check cache first. If files are already in `~/.cache/huggingface/hub`, do not re-download.

Show progress or at least log each file being downloaded.

Return local paths and pass those paths to Candle. Never pass a Hub URL to the pipeline.

Add to `.gitignore` if not already there: `*.safetensors`, `/models/`, `wallpaper-*.png`

Commands:

- `astra models download` — downloads without generating, useful for pre-warming.
- `astra generate --dry-run` — validates prompt, device, and args without downloading or running diffusion. Must succeed in CI without GPU.

## 9. Device and tier selection

Implement `src/wallpaper_generators/device.rs`:

Detect in order: Metal if compiled with `metal`, CUDA if compiled with `cuda`, else CPU.

Log the chosen device and dtype at startup.

Use sysinfo total memory to decide tier:

| Tier | Condition | Behavior |
|---|---|---|
| GPU + Turbo | GPU available and total memory >= 16 GB | Run SDXL Turbo fp16, 4 steps, full size |
| CPU fallback | No GPU, or memory < 16 GB | Log a warning, run on CPU only if `--allow-cpu` is passed, else return a friendly error suggesting GPU or smaller size |

Do not silently OOM. If device is CPU and requested size is large, warn and require `--allow-cpu`.

For unified-memory systems, total memory is the right signal. For discrete-GPU systems, use total system memory as a proxy for this task — do not attempt a separate VRAM probe unless the repo already has one.

## 10. CLI

Preserve existing CLI in `src/cli.rs` if Astra already has one. Do not create a second binary unless the repo already uses that pattern.

If starting fresh, implement with clap derive:

```bash
astra generate \
  --prompt "Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text" \
  --negative-prompt "blurry, distorted buildings, watermark, text, people, oversaturated" \
  --width 1344 --height 768 \
  --steps 4 --guidance-scale 0.0 \
  --seed 42 \
  --output wallpaper-santorini.png

astra generate --dry-run
astra models download
astra models list
```

Rules:

- `--prompt` is required for `generate` unless a default wallpaper prompt is already defined in config.
- `--negative-prompt` is optional. Turbo uses guidance 0, so negative is ignored or passed as empty. Document that.
- `--output` defaults to `wallpaper-<timestamp>.png` in current dir, or `output/` if the repo already uses that.
- `--seed` is optional. If provided, generation must be deterministic for same prompt, size, steps, and seed.
- `--help` must work without GPU and without network.
- Support env overrides: `ASTRA_OUTPUT_DIR`, `HF_HOME`, `HF_TOKEN`.

## 11. Generation pipeline — implement exactly in this order

Implement in `src/wallpaper_generators/candle_sdxl_turbo.rs`:

1. Validate: prompt non-empty, width and height divisible by 8, steps 1–10 (recommend 1–4 for Turbo), width/height <= 2048. Return clear errors, do not panic.
2. Device: call `select_device()`, set seed on device with `device.set_seed(seed)`.
3. Config + scheduler: `StableDiffusionConfig::sdxl_turbo(sliced_attention_size, Some(height), Some(width))`, then `config.build_scheduler(steps)`. Use `scheduler.timesteps()`, `scheduler.init_noise_sigma()`, `scheduler.scale_model_input()`, `scheduler.step()`.
4. Download / cache: call `ensure_model_files(use_f16)` to get local paths.
5. Text embeddings — SDXL needs BOTH CLIP encoders:
   - For each encoder: load `Tokenizer::from_file()`, get `pad_id` from `clip_config.pad_with` or fallback to `<|endoftext|>`.
   - Encode prompt, truncate token ids to 77, pad to `clip_config.max_position_embeddings` with `pad_id`.
   - Build with `stable_diffusion::build_clip_transformer(clip_config, clip_weights, device, DType::F32)` — note CLIP is built in F32 even when UNet/VAE are F16.
   - Forward to get embeddings. If `guidance_scale > 1.0`, also encode unconditional prompt and `Tensor::cat([uncond, text], 0)`, else use text embeddings only (Turbo default).
   - After both encoders, `Tensor::cat(&[emb1, emb2], D::Minus1)` — concat on last dim.
6. VAE + UNet:
   - `config.build_vae(vae_path, device, dtype)`
   - `config.build_unet(unet_path, device, 4, false, dtype)` — in_channels = 4, use_flash_attn = false for this task.
7. Initial latents: `Tensor::randn(0f32, 1f32, (1, 4, height/8, width/8), device)` then `* scheduler.init_noise_sigma()`, convert to dtype.
8. Denoising loop: for each timestep:
   - If `use_guide_scale`, `Tensor::cat([latents, latents], 0)` else clone.
   - `scheduler.scale_model_input(latent_input, timestep)`
   - `unet.forward(latent_input, timestep as f64, text_embeddings)`
   - If guided: split `noise_pred.chunk(2, 0)`, combine `uncond + (text - uncond) * guidance_scale`.
   - `scheduler.step(noise_pred, timestep, latents)`
   - Log `step i/N done in Xs`.
9. Decode + save:
   - `vae.decode(&(latents / 0.13025))`
   - `((image / 2) + 0.5)`, move to CPU, clamp 0..1, `* 255`, convert to U8, take first batch `.i(0)`.
   - Check dims are `(3, height, width)`, permute `(1,2,0)`, flatten, build `image::ImageBuffer::Rgb` from raw, save to `--output`. Create parent dir if needed.
   - Print: device, model, size, steps, seed, output path, elapsed seconds.

Error handling: use `anyhow::Result` with `.context()`. Contexts to cover:

- Missing model file → suggest `astra models download`
- Download failure → show Hub error, do not panic
- OOM / device failure → show device, dtype, size, suggest smaller size or `--allow-cpu`
- Invalid size → show valid sizes

Keep `main.rs` thin. All generation logic lives in `wallpaper_generators/`.

## 12. File layout to create / modify

Target layout — adapt to existing repo, do not duplicate:

```
src/
  main.rs                    # add: pub mod wallpaper_generators; keep thin
  cli.rs                     # add Generate + Models subcommands
  wallpaper_generators/
    mod.rs                   # pub mod candle_sdxl_turbo; pub mod device; pub mod download;
    candle_sdxl_turbo.rs     # GenerateRequest, GenerateOutput, generate(), dry_run(), validate(), text_embeddings()
    device.rs                # SelectedDevice, select_device(), pick_tier(), total_memory_gb()
    download.rs              # ModelFiles, ensure_model_files(), list_model_files(), repo constants
tests/
  cli_help.rs                # --help works
  validation.rs              # width/height/steps validation
  dry_run.rs                 # --dry-run works without GPU
README.md                    # updated
.gitignore                   # updated
```

Do not put generation logic in `main.rs`.

## 13. Tests

`cargo test` must pass on CPU in CI.

Unit tests: width/height validation, steps validation, prompt validation, tier selection given mocked memory values, cache path resolution.

Integration tests: `astra --help`, `astra generate --dry-run`, `astra models list` work without GPU or network.

Do not require GPU or a full model download in `cargo test`. Gate any real generation test behind `#[ignore]` or env var `ASTRA_RUN_GPU_TEST=1`.

Add a test that asserts no Python files are invoked by the Rust generation path.

## 14. Docs

Update README.md with:

- What this adds, and that it is pure Rust.
- Install: `cargo build --release`, plus Metal vs CUDA build commands.
- First run: `cargo run --release --features metal -- generate --prompt "..." --output wallpaper.png`
- Where models are cached, and how to set `HF_HOME`.
- Example output image, if generated locally during dev, with prompt in caption.
- Troubleshooting: OOM, missing token, Metal vs CUDA mismatch.

## 15. Acceptance criteria — all must be true before done

- `cargo build --release` succeeds without GPU features.
- `cargo build --release --features metal` succeeds on macOS, `cargo build --release --features cuda,cudnn` succeeds on Nvidia Linux/Windows (at least one verified by the agent's environment, the other documented).
- `cargo test` passes.
- `cargo fmt --check` and `cargo clippy -- -D warnings` pass.
- `astra generate --dry-run` succeeds without GPU.
- `astra generate --prompt "Santorini, Greece, whitewashed buildings with blue domes, Aegean Sea, golden hour, photorealistic photograph, no people, no text" --width 1344 --height 768 --steps 4 --output wallpaper-santorini.png` produces a valid PNG on GPU.
- Second run reuses cache, does not re-download.
- No Python subprocess is spawned by generate.
- README updated with build and usage.

## 16. Implementation order for OpenCode

1. Explore repo: read `Cargo.toml`, `src/main.rs`, `src/cli.rs`, `src/constants.rs`, existing `wallpaper_generators/`, `README.md`.
2. Add dependencies and features, get `cargo build` green on CPU.
3. Implement `device.rs` + `download.rs` + `models download` + `models list`.
4. Port Candle SDXL Turbo pipeline into `candle_sdxl_turbo.rs`, wire CLI.
5. Implement validation, logging, error messages.
6. Add tests, docs, `.gitignore`.
7. Generate one test wallpaper locally if GPU is available, attach to summary.
8. Create branch `feat/candle-sdxl-turbo`, commit with conventional messages, leave as local branch. Do not push unless explicitly asked.

## 17. Rules for the agent

- If a Candle API has changed (e.g. `build_clip_transformer` signature, `StableDiffusionConfig` fields), check the installed candle-transformers docs in `~/.cargo` or docs.rs for the pinned version — do not guess.
- If you must deviate from this spec, list deviations at the end of your summary with reason.
- Small, conventional commits. No wip commits, no model weights, no secrets.
- Do not add a web UI, scheduler, or upscaler to satisfy this spec.
