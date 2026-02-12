use std::path::Path;
use std::process::Command;

fn project_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Run `nazec test` against the example test files and verify all pass.
#[test]
fn nazec_test_examples_pass() {
    let root = project_root();
    let output = Command::new(env!("CARGO_BIN_EXE_nazec"))
        .arg("test")
        .current_dir(&root)
        .output()
        .expect("failed to run nazec test");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "nazec test failed:\n{}", stderr);
    assert!(
        stderr.contains("0 failed"),
        "expected 0 failures in output:\n{}",
        stderr
    );
}

/// Run `nazec test --format json` and verify valid JSON with 0 failures.
#[test]
fn nazec_test_json_output() {
    let root = project_root();
    let output = Command::new(env!("CARGO_BIN_EXE_nazec"))
        .args(["test", "--format", "json"])
        .current_dir(&root)
        .output()
        .expect("failed to run nazec test --format json");

    assert!(output.status.success(), "nazec test --format json failed");

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("invalid JSON output");

    assert_eq!(json["failed"], 0, "expected 0 failures in JSON output");
    assert!(
        json["total"].as_u64().unwrap() > 0,
        "expected at least 1 test"
    );
}

/// Run `nazec test --filter` and verify only matching tests run.
#[test]
fn nazec_test_filter() {
    let root = project_root();
    let output = Command::new(env!("CARGO_BIN_EXE_nazec"))
        .args(["test", "--filter", "increments"])
        .current_dir(&root)
        .output()
        .expect("failed to run nazec test --filter");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "filtered test failed:\n{}", stderr);
    assert!(
        stderr.contains("1 passed"),
        "expected exactly 1 test to run:\n{}",
        stderr
    );
}
