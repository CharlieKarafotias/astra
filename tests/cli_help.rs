use std::process::Command;

fn astra() -> Command {
    Command::new(env!("CARGO_BIN_EXE_astra"))
}

#[test]
fn help_works_without_gpu_or_network() {
    let output = astra().arg("--help").output().expect("run astra --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("generate"));
    assert!(stdout.contains("models"));
}

#[test]
fn generate_help_lists_the_sdxl_subcommand() {
    let output = astra()
        .args(["generate", "--help"])
        .output()
        .expect("run astra generate --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sdxl"));
}

#[test]
fn sdxl_help_lists_the_generation_flags() {
    let output = astra()
        .args(["generate", "sdxl", "--help"])
        .output()
        .expect("run astra generate sdxl --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for flag in [
        "--prompt",
        "--negative-prompt",
        "--width",
        "--height",
        "--steps",
        "--guidance-scale",
        "--seed",
        "--output",
        "--fast",
        "--dry-run",
        "--allow-cpu",
    ] {
        assert!(stdout.contains(flag), "help is missing {flag}:\n{stdout}");
    }
}
