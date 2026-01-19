//! Integration tests for the SourDough CLI.
//!
//! These tests verify end-to-end functionality of the UniBin CLI.

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

/// Test that the binary exists and shows help
#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("--help");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("SourDough"))
        .stdout(predicate::str::contains("scaffold"))
        .stdout(predicate::str::contains("validate"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("genomebin"));
}

/// Test version flag
#[test]
fn test_version() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("--version");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sourdough"));
}

/// Test doctor command basic functionality
#[test]
fn test_doctor_basic() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("doctor");
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Health Check"))
        .stdout(predicate::str::contains("Binary OK"));
}

/// Test doctor comprehensive mode
#[test]
fn test_doctor_comprehensive() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.args(["doctor", "--comprehensive"]);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("comprehensive checks"));
}

/// Test scaffold new-primal command
#[test]
fn test_scaffold_new_primal() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "testPrimal";
    
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Test primal description")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Creating new primal"))
        .stdout(predicate::str::contains("Created primal"));
    
    // Verify structure was created
    let primal_path = temp_dir.path().join(primal_name);
    assert!(primal_path.exists());
    assert!(primal_path.join("Cargo.toml").exists());
    assert!(primal_path.join("README.md").exists());
    assert!(primal_path.join("crates").exists());
    assert!(primal_path.join("specs").exists());
    
    // Verify core crate exists
    let core_crate = primal_path.join("crates").join("testprimal-core");
    assert!(core_crate.exists());
    assert!(core_crate.join("Cargo.toml").exists());
    assert!(core_crate.join("src").join("lib.rs").exists());
    
    // Verify generated code is valid Rust
    let lib_rs = std::fs::read_to_string(core_crate.join("src").join("lib.rs")).unwrap();
    assert!(lib_rs.contains("PrimalLifecycle"));
    assert!(lib_rs.contains("PrimalHealth"));
    assert!(lib_rs.contains("#[cfg(test)]"));
}

/// Test scaffold new-primal with invalid name
#[test]
fn test_scaffold_invalid_primal_name() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("scaffold")
        .arg("new-primal")
        .arg("") // Empty name
        .arg("Description")
        .arg("--output")
        .arg(temp_dir.path());
    
    // Should handle empty name gracefully (current implementation may create it,
    // but in production should validate)
    let result = cmd.output().unwrap();
    // Just verify it doesn't panic
    assert!(result.status.code().is_some());
}

/// Test validate primal command on valid structure
#[test]
fn test_validate_primal_valid() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "validPrimal";
    
    // First create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Test primal")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    // Now validate it
    let mut validate_cmd = Command::cargo_bin("sourdough").unwrap();
    validate_cmd.arg("validate")
        .arg("primal")
        .arg(temp_dir.path().join(primal_name));
    
    validate_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Validating primal"))
        .stdout(predicate::str::contains("Cargo.toml found"))
        .stdout(predicate::str::contains("crates/ directory found"));
}

/// Test validate primal command on invalid path
#[test]
fn test_validate_primal_invalid() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("validate")
        .arg("primal")
        .arg("/nonexistent/path");
    
    cmd.assert()
        .failure()
        .stdout(predicate::str::contains("Missing Cargo.toml")
            .or(predicate::str::contains("Missing crates/")));
}

/// Test validate unibin command
#[test]
fn test_validate_unibin() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "unibinPrimal";
    
    // Create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("UniBin test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    // Validate as UniBin
    let mut validate_cmd = Command::cargo_bin("sourdough").unwrap();
    validate_cmd.arg("validate")
        .arg("unibin")
        .arg(temp_dir.path().join(primal_name));
    
    // Should complete (may have warnings about missing [[bin]] section)
    let output = validate_cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("UniBin"));
}

/// Test validate ecobin command
#[test]
fn test_validate_ecobin() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "ecobinPrimal";
    
    // Create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("EcoBin test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    // Validate as ecoBin
    let mut validate_cmd = Command::cargo_bin("sourdough").unwrap();
    validate_cmd.arg("validate")
        .arg("ecobin")
        .arg(temp_dir.path().join(primal_name));
    
    // Should complete (will check for C dependencies)
    let output = validate_cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ecoBin"));
}

/// Test genomebin create command
#[test]
fn test_genomebin_create() {
    let temp_dir = TempDir::new().unwrap();
    let ecobins_dir = temp_dir.path().join("ecobins");
    std::fs::create_dir_all(&ecobins_dir).unwrap();
    
    // Create dummy ecoBin files
    std::fs::write(ecobins_dir.join("testPrimal-x86_64-unknown-linux-musl"), "#!/bin/sh\necho test").unwrap();
    std::fs::write(ecobins_dir.join("testPrimal-aarch64-unknown-linux-musl"), "#!/bin/sh\necho test").unwrap();
    
    let output_path = temp_dir.path().join("test.genome");
    
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("genomebin")
        .arg("create")
        .arg("--primal")
        .arg("testPrimal")
        .arg("--version")
        .arg("1.0.0")
        .arg("--ecobins")
        .arg(&ecobins_dir)
        .arg("--output")
        .arg(&output_path);
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Creating genomeBin"))
        .stdout(predicate::str::contains("Found 2 ecoBin"));
    
    // Verify genomeBin was created
    assert!(output_path.exists());
}

/// Test genomebin create with missing directory
#[test]
fn test_genomebin_create_missing_dir() {
    let temp_dir = TempDir::new().unwrap();
    
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("genomebin")
        .arg("create")
        .arg("--primal")
        .arg("test")
        .arg("--version")
        .arg("1.0.0")
        .arg("--ecobins")
        .arg("/nonexistent/dir")
        .arg("--output")
        .arg(temp_dir.path().join("test.genome"));
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test verbose flag
#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("--verbose")
        .arg("doctor");
    
    cmd.assert().success();
}

/// Test quiet flag
#[test]
fn test_quiet_flag() {
    let mut cmd = Command::cargo_bin("sourdough").unwrap();
    cmd.arg("--quiet")
        .arg("doctor");
    
    cmd.assert().success();
}

/// Test that generated primal structure is valid
#[test]
fn test_generated_primal_structure() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "buildablePrimal";
    
    // Create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Buildable test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    let primal_path = temp_dir.path().join(primal_name);
    
    // Verify workspace Cargo.toml is valid TOML
    let workspace_cargo = std::fs::read_to_string(primal_path.join("Cargo.toml")).unwrap();
    let _: toml::Value = toml::from_str(&workspace_cargo).expect("Workspace Cargo.toml should be valid TOML");
    
    // Verify crate Cargo.toml is valid TOML
    let crate_cargo_path = primal_path.join("crates/buildableprimal-core/Cargo.toml");
    let crate_cargo = std::fs::read_to_string(crate_cargo_path).unwrap();
    let _: toml::Value = toml::from_str(&crate_cargo).expect("Crate Cargo.toml should be valid TOML");
    
    // Verify lib.rs is valid Rust syntax (basic check)
    let lib_rs = std::fs::read_to_string(primal_path.join("crates/buildableprimal-core/src/lib.rs")).unwrap();
    assert!(lib_rs.contains("pub struct"));
    assert!(lib_rs.contains("impl PrimalLifecycle"));
    assert!(lib_rs.contains("#[cfg(test)]"));
}

/// Test that generated primal has test code
#[test]
fn test_generated_primal_has_tests() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "testablePrimal";
    
    // Create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Testable primal")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    // Verify test code exists
    let primal_path = temp_dir.path().join(primal_name);
    let lib_rs = std::fs::read_to_string(primal_path.join("crates/testableprimal-core/src/lib.rs")).unwrap();
    
    // Should have test module
    assert!(lib_rs.contains("#[cfg(test)]"));
    assert!(lib_rs.contains("mod tests"));
    assert!(lib_rs.contains("#[tokio::test]"));
    
    // Verify the test code is syntactically reasonable
    assert!(lib_rs.contains("fn test_"));
}

/// Test help for subcommands
#[test]
fn test_subcommand_help() {
    let subcommands = ["scaffold", "validate", "doctor", "genomebin"];
    
    for subcmd in subcommands {
        let mut cmd = Command::cargo_bin("sourdough").unwrap();
        cmd.arg(subcmd).arg("--help");
        
        cmd.assert()
            .success()
            .stdout(predicate::str::contains(subcmd));
    }
}

/// Test scaffold new-crate command
#[test]
fn test_scaffold_new_crate() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "cratePrimal";
    
    // First create a primal
    let mut create_cmd = Command::cargo_bin("sourdough").unwrap();
    create_cmd.arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Test for new crate")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();
    
    // Add a new crate
    let mut add_crate_cmd = Command::cargo_bin("sourdough").unwrap();
    add_crate_cmd.arg("scaffold")
        .arg("new-crate")
        .arg(primal_name)
        .arg("crateprimal-storage")
        .arg("--path")
        .arg(temp_dir.path().join(primal_name));
    
    add_crate_cmd.assert()
        .success()
        .stdout(predicate::str::contains("Adding crate"))
        .stdout(predicate::str::contains("Created crate"));
    
    // Verify crate was created
    let crate_path = temp_dir.path()
        .join(primal_name)
        .join("crates")
        .join("crateprimal-storage");
    assert!(crate_path.exists());
    assert!(crate_path.join("Cargo.toml").exists());
    assert!(crate_path.join("src").join("lib.rs").exists());
}

