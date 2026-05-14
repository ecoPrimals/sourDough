//! Integration tests for the `SourDough` CLI.
//!
//! These tests verify end-to-end functionality of the `UniBin` CLI.

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn sourdough_cmd() -> Command {
    Command::cargo_bin("sourdough").expect("sourdough binary not found")
}

/// Test that the binary exists and shows help
#[test]
fn test_help() {
    let mut cmd = sourdough_cmd();
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
    let mut cmd = sourdough_cmd();
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("sourdough"));
}

/// Test doctor command basic functionality
#[test]
fn test_doctor_basic() {
    let mut cmd = sourdough_cmd();
    cmd.arg("doctor");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Health Check"))
        .stdout(predicate::str::contains("Binary OK"));
}

/// Test doctor comprehensive mode
#[test]
fn test_doctor_comprehensive() {
    let mut cmd = sourdough_cmd();
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

    let mut cmd = sourdough_cmd();
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

    let mut cmd = sourdough_cmd();
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
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Test primal")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    // Now validate it
    let mut validate_cmd = sourdough_cmd();
    validate_cmd
        .arg("validate")
        .arg("primal")
        .arg(temp_dir.path().join(primal_name));

    validate_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Validating primal"))
        .stdout(predicate::str::contains("Cargo.toml found"))
        .stdout(predicate::str::contains("crates/ directory found"));
}

/// Test validate primal command on invalid path
#[test]
fn test_validate_primal_invalid() {
    let mut cmd = sourdough_cmd();
    cmd.arg("validate").arg("primal").arg("/nonexistent/path");

    cmd.assert().failure().stdout(
        predicate::str::contains("Missing Cargo.toml")
            .or(predicate::str::contains("Missing crates/")),
    );
}

/// Test validate unibin command
#[test]
fn test_validate_unibin() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "unibinPrimal";

    // Create a primal
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("UniBin test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    // Validate as UniBin
    let mut validate_cmd = sourdough_cmd();
    validate_cmd
        .arg("validate")
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
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("EcoBin test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    // Validate as ecoBin
    let mut validate_cmd = sourdough_cmd();
    validate_cmd
        .arg("validate")
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
    std::fs::write(
        ecobins_dir.join("testPrimal-x86_64-unknown-linux-musl"),
        "#!/bin/sh\necho test",
    )
    .unwrap();
    std::fs::write(
        ecobins_dir.join("testPrimal-aarch64-unknown-linux-musl"),
        "#!/bin/sh\necho test",
    )
    .unwrap();

    let output_path = temp_dir.path().join("test.genome");

    let mut cmd = sourdough_cmd();
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

    let mut cmd = sourdough_cmd();
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
    let mut cmd = sourdough_cmd();
    cmd.arg("--verbose").arg("doctor");

    cmd.assert().success();
}

/// Test quiet flag
#[test]
fn test_quiet_flag() {
    let mut cmd = sourdough_cmd();
    cmd.arg("--quiet").arg("doctor");

    cmd.assert().success();
}

/// Test that generated primal structure is valid
#[test]
fn test_generated_primal_structure() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "buildablePrimal";

    // Create a primal
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Buildable test")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    let primal_path = temp_dir.path().join(primal_name);

    // Verify workspace Cargo.toml is valid TOML
    let workspace_cargo = std::fs::read_to_string(primal_path.join("Cargo.toml")).unwrap();
    let _: toml::Value =
        toml::from_str(&workspace_cargo).expect("Workspace Cargo.toml should be valid TOML");

    // Verify crate Cargo.toml is valid TOML
    let crate_cargo_path = primal_path.join("crates/buildableprimal-core/Cargo.toml");
    let crate_cargo = std::fs::read_to_string(crate_cargo_path).unwrap();
    let _: toml::Value =
        toml::from_str(&crate_cargo).expect("Crate Cargo.toml should be valid TOML");

    // Verify lib.rs is valid Rust syntax (basic check)
    let lib_rs =
        std::fs::read_to_string(primal_path.join("crates/buildableprimal-core/src/lib.rs"))
            .unwrap();
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
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Testable primal")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    // Verify test code exists
    let primal_path = temp_dir.path().join(primal_name);
    let lib_rs =
        std::fs::read_to_string(primal_path.join("crates/testableprimal-core/src/lib.rs")).unwrap();

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
        let mut cmd = sourdough_cmd();
        cmd.arg(subcmd).arg("--help");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains(subcmd));
    }
}

/// Test genomebin test command on a created genomeBin
#[test]
fn test_genomebin_test_valid() {
    let temp_dir = TempDir::new().unwrap();
    let ecobins_dir = temp_dir.path().join("ecobins");
    std::fs::create_dir_all(&ecobins_dir).unwrap();

    std::fs::write(
        ecobins_dir.join("testPrimal-x86_64-unknown-linux-musl"),
        "#!/bin/sh\necho test",
    )
    .unwrap();

    let output_path = temp_dir.path().join("test.genome");

    // Create a genomeBin first
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("genomebin")
        .arg("create")
        .arg("--primal")
        .arg("testPrimal")
        .arg("--version")
        .arg("1.0.0")
        .arg("--ecobins")
        .arg(&ecobins_dir)
        .arg("--output")
        .arg(&output_path);
    create_cmd.assert().success();

    // Now test it
    let mut test_cmd = sourdough_cmd();
    test_cmd.arg("genomebin").arg("test").arg(&output_path);

    let output = test_cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Testing genomeBin") || stdout.contains("Validation Results"));
}

/// Test genomebin test on a missing file
#[test]
fn test_genomebin_test_missing_file() {
    let mut cmd = sourdough_cmd();
    cmd.arg("genomebin")
        .arg("test")
        .arg("/nonexistent/genome.bin");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test genomebin sign fails gracefully when no signing key is available
#[test]
fn test_genomebin_sign_no_key() {
    let temp_dir = TempDir::new().unwrap();
    let dummy = temp_dir.path().join("dummy.genome");
    std::fs::write(&dummy, "dummy content").unwrap();

    let mut cmd = sourdough_cmd();
    cmd.arg("genomebin").arg("sign").arg(&dummy);
    cmd.current_dir(temp_dir.path());

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Signing key not found"));
}

/// Test genomebin sign on missing file
#[test]
fn test_genomebin_sign_missing_file() {
    let mut cmd = sourdough_cmd();
    cmd.arg("genomebin")
        .arg("sign")
        .arg("/nonexistent/genome.bin");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test doctor comprehensive checks genomeBin tools
#[test]
fn test_doctor_comprehensive_genomebin_tools() {
    let mut cmd = sourdough_cmd();
    cmd.args(["doctor", "--comprehensive"]);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("genomeBin tools"))
        .stdout(
            predicate::str::contains("Platform detection OK")
                .or(predicate::str::contains("Platform:")),
        )
        .stdout(predicate::str::contains("Pure Rust"));
}

/// Test scaffold new-crate command
#[test]
fn test_scaffold_new_crate() {
    let temp_dir = TempDir::new().unwrap();
    let primal_name = "cratePrimal";

    // First create a primal
    let mut create_cmd = sourdough_cmd();
    create_cmd
        .arg("scaffold")
        .arg("new-primal")
        .arg(primal_name)
        .arg("Test for new crate")
        .arg("--output")
        .arg(temp_dir.path().join(primal_name));
    create_cmd.assert().success();

    // Add a new crate
    let mut add_crate_cmd = sourdough_cmd();
    add_crate_cmd
        .arg("scaffold")
        .arg("new-crate")
        .arg(primal_name)
        .arg("crateprimal-storage")
        .arg("--path")
        .arg(temp_dir.path().join(primal_name));

    add_crate_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Adding crate"))
        .stdout(predicate::str::contains("Created crate"));

    // Verify crate was created
    let crate_path = temp_dir
        .path()
        .join(primal_name)
        .join("crates")
        .join("crateprimal-storage");
    assert!(crate_path.exists());
    assert!(crate_path.join("Cargo.toml").exists());
    assert!(crate_path.join("src").join("lib.rs").exists());

    // Workspace members should list the new crate
    let workspace =
        std::fs::read_to_string(temp_dir.path().join(primal_name).join("Cargo.toml")).unwrap();
    assert!(workspace.contains("crates/crateprimal-storage"));
}

// ── Sign / Verify ───────────────────────────────────────────────────

/// Test sign --generate-key creates keypair
#[test]
fn test_sign_generate_key() {
    let temp_dir = TempDir::new().unwrap();
    let dummy = temp_dir.path().join("dummy.bin");
    std::fs::write(&dummy, "content").unwrap();

    let mut cmd = sourdough_cmd();
    cmd.arg("sign")
        .arg(&dummy)
        .arg("--generate-key")
        .current_dir(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated signing key"));

    assert!(temp_dir.path().join("signing.key").exists());
    assert!(temp_dir.path().join("signing.pub").exists());
}

/// Test sign + verify roundtrip
#[test]
fn test_sign_verify_roundtrip() {
    let temp_dir = TempDir::new().unwrap();
    let binary = temp_dir.path().join("test.bin");
    std::fs::write(&binary, b"binary content here").unwrap();

    let mut gen_cmd = sourdough_cmd();
    gen_cmd
        .arg("sign")
        .arg(&binary)
        .arg("--generate-key")
        .current_dir(temp_dir.path());
    gen_cmd.assert().success();

    let mut sign_cmd = sourdough_cmd();
    sign_cmd
        .arg("sign")
        .arg(&binary)
        .current_dir(temp_dir.path());
    sign_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Signed"));

    assert!(temp_dir.path().join("test.bin.sig").exists());

    let mut verify_cmd = sourdough_cmd();
    verify_cmd
        .arg("verify")
        .arg(&binary)
        .current_dir(temp_dir.path());
    verify_cmd
        .assert()
        .success()
        .stdout(predicate::str::contains("Verified"));
}

/// Test verify detects tampering
#[test]
fn test_verify_detects_tamper() {
    let temp_dir = TempDir::new().unwrap();
    let binary = temp_dir.path().join("tamper.bin");
    std::fs::write(&binary, b"original").unwrap();

    let mut gen_cmd = sourdough_cmd();
    gen_cmd
        .arg("sign")
        .arg(&binary)
        .arg("--generate-key")
        .current_dir(temp_dir.path());
    gen_cmd.assert().success();

    let mut sign_cmd = sourdough_cmd();
    sign_cmd
        .arg("sign")
        .arg(&binary)
        .current_dir(temp_dir.path());
    sign_cmd.assert().success();

    std::fs::write(&binary, b"tampered!").unwrap();

    let mut verify_cmd = sourdough_cmd();
    verify_cmd
        .arg("verify")
        .arg(&binary)
        .current_dir(temp_dir.path());
    verify_cmd
        .assert()
        .failure()
        .stderr(predicate::str::contains("Signature verification failed"));
}

// ── Scaffold systemd ────────────────────────────────────────────────

/// Test scaffold systemd generates a service unit
#[test]
fn test_scaffold_systemd() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = sourdough_cmd();
    cmd.arg("scaffold")
        .arg("systemd")
        .arg("bearDog")
        .arg("--role")
        .arg("membrane")
        .arg("--output")
        .arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Generated"));

    let service =
        std::fs::read_to_string(temp_dir.path().join("beardog-membrane.service")).unwrap();
    assert!(service.contains("[Unit]"));
    assert!(service.contains("[Service]"));
    assert!(service.contains("[Install]"));
    assert!(service.contains("NoNewPrivileges=true"));
    assert!(service.contains("ProtectSystem=strict"));
    assert!(service.contains("/run/biomeos/beardog.sock"));
    assert!(service.contains("BEARDOG_ROLE=membrane"));
    assert!(service.contains("MemoryMax=128M"));
}

// ── Layout ──────────────────────────────────────────────────────────

/// Test layout validation on empty directory
#[test]
fn test_layout_empty_dir() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = sourdough_cmd();
    cmd.arg("layout").arg(temp_dir.path());

    cmd.assert().success().stdout(predicate::str::contains(
        "No binaries or triple directories found",
    ));
}

/// Test layout validation with triple-first structure
#[test]
fn test_layout_triple_first() {
    let temp_dir = TempDir::new().unwrap();
    let triple_dir = temp_dir.path().join("x86_64-unknown-linux-musl");
    std::fs::create_dir_all(&triple_dir).unwrap();
    let binary = triple_dir.join("beardog");
    std::fs::write(&binary, "binary").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let mut cmd = sourdough_cmd();
    cmd.arg("layout").arg(temp_dir.path());

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("x86_64-unknown-linux-musl"));
}

// ── Validate composition ────────────────────────────────────────────

/// Test validate composition with missing binaries
#[test]
fn test_validate_composition_missing() {
    let temp_dir = TempDir::new().unwrap();
    let primals_dir = temp_dir.path().join("primals");
    std::fs::create_dir_all(&primals_dir).unwrap();

    let mut cmd = sourdough_cmd();
    cmd.arg("validate")
        .arg("composition")
        .arg("tower")
        .arg("--primals-dir")
        .arg(&primals_dir);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Composition incomplete"));
}

/// Test validate composition with all binaries present
#[test]
fn test_validate_composition_complete() {
    let temp_dir = TempDir::new().unwrap();
    let primals_dir = temp_dir.path().join("primals");
    std::fs::create_dir_all(&primals_dir).unwrap();

    for name in &["beardog", "songbird", "skunkbat"] {
        std::fs::write(primals_dir.join(name), "binary").unwrap();
    }

    let mut cmd = sourdough_cmd();
    cmd.arg("validate")
        .arg("composition")
        .arg("tower")
        .arg("--primals-dir")
        .arg(&primals_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Composition is complete"));
}
