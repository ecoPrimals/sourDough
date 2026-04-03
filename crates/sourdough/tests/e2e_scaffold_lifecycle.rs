//! End-to-end test: full primal scaffold lifecycle.
//!
//! Verifies that a scaffolded primal builds, tests, and validates
//! without any dependency on sourDough at compile time or runtime.

use std::process::Command;
use tempfile::TempDir;

/// Full lifecycle: scaffold -> build -> test -> validate.
///
/// This is the canonical e2e test for sourDough's primary purpose.
#[test]
fn scaffold_build_test_validate() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "e2ePrimal";
    let primal_path = temp_dir.path().join(primal_name);

    // 1. Scaffold
    let sourdough = env!("CARGO_BIN_EXE_sourdough");
    let output = Command::new(sourdough)
        .args([
            "scaffold",
            "new-primal",
            primal_name,
            "E2E test primal",
            "--output",
        ])
        .arg(&primal_path)
        .output()
        .expect("scaffold command failed");
    assert!(
        output.status.success(),
        "scaffold failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 2. Build the scaffolded primal (proves self-containment)
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(&primal_path)
        .output()
        .expect("cargo build failed");
    assert!(
        output.status.success(),
        "scaffolded primal cargo build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 3. Run tests on the scaffolded primal
    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(&primal_path)
        .output()
        .expect("cargo test failed");
    assert!(
        output.status.success(),
        "scaffolded primal cargo test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 4. Validate the scaffolded primal
    let output = Command::new(sourdough)
        .args(["validate", "primal"])
        .arg(&primal_path)
        .output()
        .expect("validate command failed");
    assert!(
        output.status.success(),
        "validate failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 5. Verify no sourDough dependency in generated code
    let cargo_toml = std::fs::read_to_string(primal_path.join("Cargo.toml")).unwrap();
    assert!(
        !cargo_toml.contains("sourdough-core"),
        "scaffolded primal must not depend on sourdough-core"
    );

    let core_cargo = std::fs::read_to_string(
        primal_path
            .join("crates")
            .join("e2eprimal-core")
            .join("Cargo.toml"),
    )
    .unwrap();
    assert!(
        !core_cargo.contains("sourdough"),
        "scaffolded core crate must not reference sourdough"
    );
}

/// Scaffold a primal, add a crate, then build the full workspace.
#[test]
fn scaffold_add_crate_build() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "multiCrate";
    let primal_path = temp_dir.path().join(primal_name);

    let sourdough = env!("CARGO_BIN_EXE_sourdough");

    // 1. Scaffold base primal
    let output = Command::new(sourdough)
        .args([
            "scaffold",
            "new-primal",
            primal_name,
            "Multi-crate test",
            "--output",
        ])
        .arg(&primal_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    // 2. Add a second crate
    let output = Command::new(sourdough)
        .args([
            "scaffold",
            "new-crate",
            primal_name,
            "multicrate-storage",
            "--path",
        ])
        .arg(&primal_path)
        .output()
        .unwrap();
    assert!(output.status.success());

    // 3. Build the whole workspace
    let output = Command::new("cargo")
        .args(["build"])
        .current_dir(&primal_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "multi-crate build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // 4. Test the whole workspace
    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(&primal_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "multi-crate test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
