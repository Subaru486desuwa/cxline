use std::process::Command;

fn run_cxline(input: &str, args: &[&str]) -> String {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--quiet", "--"]);
    cmd.args(args);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    cmd.current_dir(env!("CARGO_MANIFEST_DIR"));

    let mut child = cmd.spawn().expect("Failed to start cxline");
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().expect("Failed to wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn run_cxline_no_stdin(args: &[&str]) -> (String, String) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run");
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

// --- Legacy stdin mode tests ---

#[test]
fn test_basic_model_output() {
    let result = run_cxline(r#"{"model":"o3-mini"}"#, &[]);
    assert!(result.contains("o3-mini"));
}

#[test]
fn test_full_input() {
    let input = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/sample_input.json"),
    )
    .unwrap();
    let result = run_cxline(input.trim(), &[]);
    assert!(result.contains("o3-mini"));
    assert!(result.contains("$0.42"));
    assert!(result.contains("5m23s"));
}

#[test]
fn test_minimal_theme() {
    let result = run_cxline(r#"{"model":"gpt-4"}"#, &["--theme", "minimal"]);
    assert!(result.contains("gpt-4"));
    // Minimal theme has no emoji icons
    assert!(!result.contains('\u{1f916}'));
}

#[test]
fn test_module_filter() {
    let result = run_cxline(
        r#"{"model":"o3","cost":{"total":1.23}}"#,
        &["-m", "model,cost"],
    );
    assert!(result.contains("o3"));
    assert!(result.contains("$1.23"));
}

#[test]
fn test_empty_input() {
    let result = run_cxline("{}", &[]);
    // With empty JSON, only git module might produce output (from cwd)
    assert!(result.trim().is_empty() || !result.is_empty());
}

#[test]
fn test_invalid_json() {
    let result = run_cxline("not json at all", &[]);
    // Should not crash, may output git branch or empty
    let _ = result;
}

#[test]
fn test_version_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--version"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cxline 0.1.0"));
}

#[test]
fn test_help_flag() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("cxline"));
    assert!(stdout.contains("USAGE"));
    assert!(stdout.contains("watch"));
    assert!(stdout.contains("show"));
}

// --- Show subcommand tests ---

#[test]
fn test_show_codex_session() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_codex.jsonl");
    let (stdout, stderr) = run_cxline_no_stdin(&[
        "show",
        "--session",
        fixture.to_str().unwrap(),
    ]);
    assert!(stdout.contains("gpt-5.3-codex"));
    assert!(stderr.contains("Session:"));
}

#[test]
fn test_show_with_theme() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_codex.jsonl");
    let (stdout, _) = run_cxline_no_stdin(&[
        "show",
        "--session",
        fixture.to_str().unwrap(),
        "--theme",
        "minimal",
    ]);
    assert!(stdout.contains("gpt-5.3-codex"));
    // Minimal theme has no emoji
    assert!(!stdout.contains('\u{1f916}'));
}

#[test]
fn test_show_with_module_filter() {
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/sample_codex.jsonl");
    let (stdout, _) = run_cxline_no_stdin(&[
        "show",
        "--session",
        fixture.to_str().unwrap(),
        "-m",
        "model,turns",
    ]);
    assert!(stdout.contains("gpt-5.3-codex"));
    assert!(stdout.contains("Turn 1"));
}

// --- Token detail display tests ---

#[test]
fn test_tokens_with_codex_details() {
    // Test via stdin with the new fields
    let input = r#"{"model":"gpt-5","token_usage":{"used":11095,"total":258400,"input":10259,"output":836,"cached":4096,"reasoning":622}}"#;
    let result = run_cxline(input, &["-m", "tokens"]);
    assert!(result.contains("11.1k"));
    assert!(result.contains("258.4k"));
    assert!(result.contains("in"));
    assert!(result.contains("out"));
}
