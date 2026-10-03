# AGENTS.md

Instructions for AI coding agents working in this repository.

## Project

Astra is a Rust command-line tool that generates high-quality wallpapers
(fractal images, spotlight images) and sets them as the desktop wallpaper.
It is cross-platform (Linux, macOS, Windows).

## Commands

Standard Rust workflow (edition 2024):

- `cargo build` — build the binary
- `cargo run -- <args>` — run the CLI locally
- `cargo test` — run the test suite (`tempfile` is available for filesystem tests)
- `cargo clippy --all-targets -- -D warnings` — lint; keep this clean
- `cargo fmt --check` — check formatting (`cargo fmt` to fix)

## Layout

- `src/main.rs` — binary entry point
- `src/cli.rs` — CLI definition (clap; shell completions via `clap_complete`)
- `src/configuration/` — user configuration loading and handling
- `src/wallpaper_generators/` — image generation backends
- `src/themes/` — wallpaper themes
- `src/os_implementations/` — per-OS wallpaper-setting code
- `src/constants.rs` — shared constants

## Platform Notes

- OS-specific code is `cfg`-gated (`windows`, `macos`, default Linux) and isolated
  in `src/os_implementations/`. Keep it that way — don't leak platform
  conditionals into shared code.
- Windows-only and macOS-only dependencies won't compile on other platforms;
  that's expected.

## Releases

- `Cargo.toml` (`[package] version`) is the single source of truth for the version.
- Pushing to `main` triggers `.github/workflows/release-and-update.yml`, which tags
  `v<VERSION>`, creates a GitHub release, and dispatches a Homebrew formula update.
  Only bump the version when you intend to cut a release.

## Conventions

- Idiomatic Rust: prefer `Result`/`Option` over panics; propagate errors with `?`.
- `cargo fmt` formatting is required.
- Address all clippy warnings; don't add `#[allow(...)]` without an explanatory comment.
- Add or extend tests for new generators, themes, and configuration behavior.
- Keep CLI `--help` text terse and consistent with existing commands.
