use std::process::Command;

fn astra() -> Command {
    Command::new(env!("CARGO_BIN_EXE_astra"))
}

#[test]
fn dry_run_succeeds_without_a_gpu_or_network() {
    let output = astra()
        .args([
            "generate",
            "sdxl",
            "--dry-run",
            "--prompt",
            "a quiet mountain lake at golden hour",
            "--width",
            "512",
            "--height",
            "512",
            "--steps",
            "2",
        ])
        .output()
        .expect("run astra generate sdxl --dry-run");

    assert!(
        output.status.success(),
        "dry run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Dry run"));
    assert!(stdout.contains("Device:"));
    assert!(stdout.contains("Model cache:"));
}

#[test]
fn dry_run_does_not_require_allow_cpu() {
    // CI machines have no GPU; dry-run must still validate successfully.
    let output = astra()
        .args([
            "generate",
            "sdxl",
            "--dry-run",
            "--prompt",
            "test",
            "--width",
            "256",
            "--height",
            "256",
        ])
        .output()
        .expect("run astra generate sdxl --dry-run");

    assert!(output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("--allow-cpu is required"));
}

#[test]
fn models_list_works_offline() {
    let output = astra()
        .args(["models", "list"])
        .output()
        .expect("run astra models list");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hugging Face cache"));
    assert!(stdout.contains("SDXL Turbo UNet"));
}

#[test]
fn fast_preset_defaults_to_three_steps() {
    let output = astra()
        .args([
            "generate",
            "sdxl",
            "--dry-run",
            "--fast",
            "--prompt",
            "test",
            "--width",
            "512",
            "--height",
            "512",
        ])
        .output()
        .expect("run astra generate sdxl --dry-run --fast");

    assert!(
        output.status.success(),
        "fast dry run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Preset: fast"));
    assert!(
        stdout.contains("steps: 3"),
        "--fast should default to 3 steps:\n{stdout}"
    );
}

#[test]
fn explicit_steps_override_the_fast_preset() {
    let output = astra()
        .args([
            "generate",
            "sdxl",
            "--dry-run",
            "--fast",
            "--steps",
            "2",
            "--prompt",
            "test",
            "--width",
            "512",
            "--height",
            "512",
        ])
        .output()
        .expect("run astra generate sdxl --dry-run --fast --steps 2");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("steps: 2"));
}
