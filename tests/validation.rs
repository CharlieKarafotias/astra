use std::process::Command;

fn dry_run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_astra"))
        .args(["generate", "sdxl", "--dry-run", "--prompt", "test"])
        .args(args)
        .output()
        .expect("run astra generate sdxl --dry-run")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn rejects_a_width_that_is_not_a_multiple_of_eight() {
    let output = dry_run(&["--width", "1001", "--height", "768"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("multiples of 8"));
}

#[test]
fn rejects_oversized_dimensions() {
    let output = dry_run(&["--width", "4096", "--height", "768"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("at most 2048"));
}

#[test]
fn rejects_zero_steps() {
    let output = dry_run(&["--width", "512", "--height", "512", "--steps", "0"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("steps must be between 1 and 10"));
}

#[test]
fn rejects_more_than_ten_steps() {
    let output = dry_run(&["--width", "512", "--height", "512", "--steps", "11"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("steps must be between 1 and 10"));
}

#[test]
fn rejects_only_one_dimension() {
    let output = dry_run(&["--width", "512"]);
    assert!(!output.status.success());
    assert!(stderr(&output).contains("pass both --width and --height"));
}

#[test]
fn rejects_an_empty_prompt() {
    let output = Command::new(env!("CARGO_BIN_EXE_astra"))
        .args([
            "generate",
            "sdxl",
            "--dry-run",
            "--prompt",
            "   ",
            "--width",
            "512",
            "--height",
            "512",
        ])
        .output()
        .expect("run astra generate sdxl --dry-run");
    assert!(!output.status.success());
    assert!(stderr(&output).contains("prompt is empty"));
}
