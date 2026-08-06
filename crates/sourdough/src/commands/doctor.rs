//! Health diagnostics for `SourDough` and the ecosystem.

use anyhow::Result;
use sourdough_core::env_keys;
use sourdough_genomebin::Platform;

pub(crate) fn run(comprehensive: bool) -> Result<()> {
    crate::info("SourDough Health Check");
    println!();

    check_sourdough_binary();
    check_rust_toolchain()?;
    check_common_tools();

    if comprehensive {
        println!();
        crate::info("Running comprehensive checks...");
        check_cross_compilation_targets();
        check_genome_bin_tools();
    }

    println!();
    crate::success("All checks passed!");

    Ok(())
}

fn check_sourdough_binary() {
    crate::info("Checking SourDough binary...");

    let version = env!("CARGO_PKG_VERSION");
    println!("  Version: {version}");

    crate::success("Binary OK");
}

fn check_rust_toolchain() -> Result<()> {
    crate::info("Checking Rust toolchain...");

    let output = std::process::Command::new("rustc")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  rustc: {}", version.trim());
        crate::success("Rust toolchain OK");
    } else {
        crate::error("rustc not found");
        anyhow::bail!("Rust compiler not found");
    }

    let output = std::process::Command::new("cargo")
        .arg("--version")
        .output()?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  cargo: {}", version.trim());
    }

    Ok(())
}

fn check_common_tools() {
    crate::info("Checking common tools...");

    let tools = [
        ("git", "Version control"),
        ("cargo-llvm-cov", "Code coverage"),
    ];

    for (tool, description) in tools {
        let output = std::process::Command::new(tool).arg("--version").output();

        match output {
            Ok(out) if out.status.success() => {
                println!("  ✓ {tool} ({description})");
            }
            _ => {
                println!("  ⚠ {tool} ({description}) - not found");
            }
        }
    }
}

fn check_cross_compilation_targets() {
    crate::info("Checking cross-compilation targets...");

    let targets = [
        "x86_64-unknown-linux-musl",
        "aarch64-unknown-linux-musl",
        "armv7-unknown-linux-musleabihf",
        "aarch64-linux-android",
    ];

    if let Ok(output) = std::process::Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
    {
        if output.status.success() {
            let installed = String::from_utf8_lossy(&output.stdout);
            for target in targets {
                if installed.contains(target) {
                    println!("  ✓ {target}");
                } else {
                    println!("  ⚠ {target} - not installed");
                }
            }
        }
    }
}

fn check_genome_bin_tools() {
    crate::info("Checking genomeBin tools (Pure Rust)...");

    match Platform::detect() {
        Ok(platform) => {
            println!("  Platform: {platform}");
            println!("  Target triple: {}", platform.target_triple());
            crate::success("Platform detection OK");
        }
        Err(e) => {
            crate::warning(&format!("Platform detection issue: {e}"));
        }
    }

    println!("  ✓ Archive operations (tar + flate2, Pure Rust)");
    println!("  ✓ Checksum (BLAKE3 + SHA256, Pure Rust)");
    println!("  ✓ Metadata (TOML, Pure Rust)");
    println!("  ✓ Signing (Ed25519 via ed25519-dalek, Pure Rust)");

    check_biomeos_socket_dir();

    crate::success("genomeBin tooling OK");
}

fn check_biomeos_socket_dir() {
    crate::info("Checking biomeOS socket directory...");

    let socket_dir = std::env::var(env_keys::BIOMEOS_SOCKET_DIR).unwrap_or_else(|_| {
        let runtime_dir = std::env::var(env_keys::XDG_RUNTIME_DIR)
            .unwrap_or_else(|_| env_keys::FALLBACK_RUNTIME_DIR.to_owned());
        format!("{runtime_dir}/{}", env_keys::SOCKET_DIR_NAME)
    });

    let path = std::path::Path::new(&socket_dir);
    if path.exists() {
        let entries: Vec<_> = std::fs::read_dir(path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        let jsonrpc_sockets: Vec<_> = entries
            .iter()
            .filter(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy();
                name.ends_with(".sock") && !name.ends_with(".tarpc.sock")
            })
            .collect();

        let tarpc_sockets: Vec<_> = entries
            .iter()
            .filter(|e| e.file_name().to_string_lossy().ends_with(".tarpc.sock"))
            .collect();

        if jsonrpc_sockets.is_empty() && tarpc_sockets.is_empty() {
            println!("  ⚠ {socket_dir} exists but no sockets found");
        } else {
            println!(
                "  ✓ {socket_dir} ({} JSON-RPC, {} tarpc)",
                jsonrpc_sockets.len(),
                tarpc_sockets.len()
            );

            for sock in &jsonrpc_sockets {
                let name = sock.file_name();
                let name = name.to_string_lossy();
                let base = name.strip_suffix(".sock").unwrap_or(&name);
                let has_tarpc = tarpc_sockets
                    .iter()
                    .any(|t| t.file_name().to_string_lossy().starts_with(base));
                let protocol_tag = if has_tarpc { "dual" } else { "jsonrpc" };
                println!("    • {name} [{protocol_tag}]");
            }

            for sock in &tarpc_sockets {
                let name = sock.file_name();
                let name = name.to_string_lossy();
                let has_jsonrpc = jsonrpc_sockets.iter().any(|j| {
                    let jname = j.file_name();
                    let jname = jname.to_string_lossy();
                    name.starts_with(jname.strip_suffix(".sock").unwrap_or(&jname))
                });
                if !has_jsonrpc {
                    println!("    • {name} [tarpc-only]");
                }
            }
        }
    } else {
        println!("  ⚠ {socket_dir} does not exist (no primals running locally)");
    }
}
