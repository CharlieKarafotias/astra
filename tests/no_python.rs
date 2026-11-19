//! The local generation path must be pure Rust: no Python files and no subprocesses.

use std::fs;
use std::path::PathBuf;

fn generation_sources() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let dir = root.join("src/wallpaper_generators");
    [
        "candle_sdxl_turbo.rs",
        "download.rs",
        "device.rs",
        "prompt.rs",
    ]
    .iter()
    .map(|name| dir.join(name))
    .collect()
}

#[test]
fn generation_path_does_not_invoke_python() {
    for path in generation_sources() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        for needle in ["python", "Python3", "python3"] {
            assert!(
                !source.contains(needle),
                "{} contains a Python reference ({needle}); the generation path must be pure Rust",
                path.display()
            );
        }
    }
}

#[test]
fn generation_path_does_not_spawn_subprocesses() {
    for path in generation_sources() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("could not read {}: {e}", path.display()));
        assert!(
            !source.contains("std::process::Command") && !source.contains("process::Command"),
            "{} spawns a subprocess; the generation path must stay in-process",
            path.display()
        );
    }
}
