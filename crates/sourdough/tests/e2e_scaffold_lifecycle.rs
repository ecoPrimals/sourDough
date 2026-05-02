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

    // 6. Verify v0.2.0 scaffold artifacts
    verify_v020_artifacts(&primal_path, &core_cargo);
}

fn verify_v020_artifacts(primal_path: &std::path::Path, core_cargo: &str) {
    let deny_toml = std::fs::read_to_string(primal_path.join("deny.toml")).unwrap();
    assert!(
        deny_toml.contains("openssl-sys"),
        "deny.toml must ban openssl-sys"
    );
    assert!(
        deny_toml.contains("[licenses]"),
        "deny.toml must have license config"
    );

    let ci_yml = std::fs::read_to_string(primal_path.join(".github/workflows/ci.yml")).unwrap();
    assert!(ci_yml.contains("cargo clippy"), "ci.yml must run clippy");
    assert!(
        ci_yml.contains("cargo deny check"),
        "ci.yml must enforce supply chain audit"
    );
    assert!(ci_yml.contains("cargo test"), "ci.yml must run tests");

    assert!(
        deny_toml.contains(r#"wrappers = ["blake3", "iana-time-zone-haiku"]"#),
        "deny.toml must allow cc as wrapper for blake3"
    );

    let notify_yml =
        std::fs::read_to_string(primal_path.join(".github/workflows/notify-plasmidbin.yml"))
            .unwrap();
    assert!(
        notify_yml.contains("primal-updated"),
        "notify-plasmidbin.yml must dispatch primal-updated event"
    );

    let server_cargo =
        std::fs::read_to_string(primal_path.join("crates/e2eprimal-server/Cargo.toml")).unwrap();
    assert!(
        server_cargo.contains("e2eprimal-core"),
        "server crate must depend on core crate"
    );

    assert!(
        core_cargo.contains("[lints]"),
        "core crate must have [lints] section for workspace inheritance"
    );

    let dispatch =
        std::fs::read_to_string(primal_path.join("crates/e2eprimal-server/src/dispatch.rs"))
            .unwrap();
    assert!(
        dispatch.contains("health.liveness"),
        "dispatch must handle health.liveness"
    );
    assert!(
        dispatch.contains("capabilities.list"),
        "dispatch must handle capabilities.list"
    );
    assert!(
        dispatch.contains("btsp.negotiate"),
        "dispatch must handle btsp.negotiate"
    );

    let server =
        std::fs::read_to_string(primal_path.join("crates/e2eprimal-server/src/server.rs")).unwrap();
    assert!(
        server.contains("fill_buf"),
        "server must implement first-byte peek"
    );
    assert!(
        server.contains("biomeos"),
        "server must use biomeos socket directory"
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
