//! Scaffolding commands for creating new primals and crates.
//!
//! sourDough is the nascent primal — the budding primal. When it scaffolds
//! a new primal, the offspring is fully self-contained: all primal DNA
//! (traits, types, patterns) is inlined. No runtime dependency on sourDough.

mod generators;
mod templates;

use anyhow::{Context, Result};
use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand)]
pub(crate) enum ScaffoldCommand {
    /// Create a new primal
    #[command(name = "new-primal")]
    NewPrimal {
        /// Name of the primal (e.g., "myPrimal")
        name: String,

        /// Description of the primal
        description: String,

        /// Output directory (defaults to parent of current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate a hardened systemd `.service` unit for a primal
    #[command(name = "systemd")]
    Systemd {
        /// Name of the primal (e.g., "bearDog")
        name: String,

        /// Deployment role (e.g., "membrane", "gate", "nest")
        #[arg(long, default_value = "gate")]
        role: String,

        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Add a new crate to an existing primal
    #[command(name = "new-crate")]
    NewCrate {
        /// Name of the primal
        primal: String,

        /// Name of the new crate (e.g., "myprimal-storage")
        crate_name: String,

        /// Path to the primal directory
        #[arg(short, long)]
        path: Option<PathBuf>,
    },

    /// Generate a standalone transport module for an existing primal
    #[command(name = "transport-kit")]
    TransportKit {
        /// Name of the primal (for module naming)
        name: String,

        /// Output directory (defaults to current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

pub(crate) fn run(cmd: ScaffoldCommand) -> Result<()> {
    match cmd {
        ScaffoldCommand::NewPrimal {
            name,
            description,
            output,
        } => create_primal(&name, &description, output),
        ScaffoldCommand::Systemd { name, role, output } => create_systemd(&name, &role, output),
        ScaffoldCommand::NewCrate {
            primal,
            crate_name,
            path,
        } => create_crate(&primal, &crate_name, path),
        ScaffoldCommand::TransportKit { name, output } => create_transport_kit(&name, output),
    }
}

fn create_primal(name: &str, description: &str, output: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Creating new primal: {name}"));

    let output_dir = output.unwrap_or_else(|| PathBuf::from("..").join(name));
    std::fs::create_dir_all(&output_dir).context("Failed to create primal directory")?;

    let crates_dir = output_dir.join("crates");
    std::fs::create_dir_all(&crates_dir)?;

    generators::write_workspace_cargo_toml(&output_dir, name)?;
    generators::create_core_crate(&crates_dir, name)?;
    generators::create_server_crate(&crates_dir, name)?;
    generators::write_deny_toml(&output_dir)?;
    generators::write_github_workflows(&output_dir, name)?;
    generators::write_capability_registry(&output_dir, name)?;
    generators::write_specs_directory(&output_dir, name, description)?;
    generators::write_readme(&output_dir, name, description)?;
    generators::write_conventions(&output_dir)?;

    crate::success(&format!(
        "Created primal '{name}' at {}",
        output_dir.display()
    ));
    crate::info("Next steps:");
    println!("  cd {}", output_dir.display());
    println!("  cargo build");
    println!("  cargo test");

    Ok(())
}

fn create_systemd(name: &str, role: &str, output: Option<PathBuf>) -> Result<()> {
    let name_lower = name.to_lowercase();
    let service_name = format!("{name_lower}-{role}.service");

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    let service_content = templates::systemd_service(name, role);
    let service_path = output_dir.join(&service_name);
    std::fs::write(&service_path, service_content)?;

    crate::success(&format!("Generated: {}", service_path.display()));
    crate::info("Install:");
    println!("  sudo cp {service_name} /etc/systemd/system/");
    println!("  sudo systemctl daemon-reload");
    println!("  sudo systemctl enable --now {service_name}");

    Ok(())
}

fn create_crate(primal: &str, crate_name: &str, path: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Adding crate '{crate_name}' to primal '{primal}'"));

    let primal_dir = path.unwrap_or_else(|| PathBuf::from("..").join(primal));

    if !primal_dir.exists() {
        anyhow::bail!("Primal directory not found: {}", primal_dir.display());
    }

    let crate_dir = primal_dir.join("crates").join(crate_name);
    let src_dir = crate_dir.join("src");
    std::fs::create_dir_all(&src_dir)?;

    let core_crate = format!("{}-core", primal.to_lowercase());
    let core_crate_ident = core_crate.replace('-', "_");

    std::fs::write(
        crate_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{crate_name}"
description = "{crate_name} crate"
version.workspace = true
edition.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true

[lints]
workspace = true

[dependencies]
{core_crate} = {{ path = "../{core_crate}" }}
tokio = {{ workspace = true }}
serde = {{ workspace = true }}
thiserror = {{ workspace = true }}
"#,
        ),
    )?;

    std::fs::write(
        src_dir.join("lib.rs"),
        format!(
            r"//! # {crate_name}
//!
//! Part of the {primal} primal.

pub use {core_crate_ident}::PrimalError;
",
        ),
    )?;

    generators::update_workspace_members(&primal_dir, crate_name)?;

    crate::success(&format!("Created crate '{crate_name}'"));
    crate::info("Workspace members updated in Cargo.toml.");

    Ok(())
}

fn create_transport_kit(name: &str, output: Option<PathBuf>) -> Result<()> {
    crate::info(&format!("Generating transport kit for: {name}"));

    let output_dir = output.unwrap_or_else(|| PathBuf::from("."));
    std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    let transport_path = output_dir.join("transport.rs");
    std::fs::write(&transport_path, transport_kit_rs(name))?;

    crate::success(&format!("Generated: {}", transport_path.display()));
    crate::info("Usage:");
    println!("  1. Copy transport.rs into your types/core crate src/");
    println!("  2. Add `pub mod transport;` to your lib.rs");
    println!("  3. Add to Cargo.toml: serde, serde_json, tokio (net feature)");
    println!("  4. Accept TRANSPORT_ENDPOINT env var in main.rs");
    println!("  5. Remove any sourdough-core dependency");
    println!();
    crate::info("Wire format (contract with songbird + ecosystem):");
    println!(
        "  {{\"transport\":\"uds\",\"path\":\"/run/user/1000/biomeos/{}.sock\"}}",
        name.to_lowercase()
    );
    println!("  {{\"transport\":\"tcp\",\"host\":\"127.0.0.1\",\"port\":9100}}");
    println!("  {{\"transport\":\"mesh_relay\",\"peer_id\":\"gate\",\"capability\":\"cap\"}}");

    Ok(())
}

#[expect(clippy::too_many_lines, reason = "static template string")]
fn transport_kit_rs(name: &str) -> String {
    let name_lower = name.to_lowercase();
    format!(
        r##"//! Transport injection types for {name}.
//!
//! Wire-compatible with sourDough `TransportEndpoint` (same serde tagged format).
//! This module is self-contained — no runtime dependency on sourDough.
//!
//! Reference: sourDough/crates/sourdough-core/src/transport.rs

use serde::{{Deserialize, Serialize}};
use tokio::net::{{TcpStream, UnixStream}};

/// How to reach a primal — the canonical transport descriptor.
///
/// Launchers inject this via `TRANSPORT_ENDPOINT` env var as JSON.
/// The primal never decides its own transport.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "transport", rename_all = "snake_case")]
pub enum TransportEndpoint {{
    /// Unix domain socket (local, lowest latency).
    Uds {{ path: String }},
    /// TCP socket (LAN or remote).
    Tcp {{ host: String, port: u16 }},
    /// Mesh relay via songbird (cross-gate).
    MeshRelay {{ peer_id: String, capability: String }},
}}

impl TransportEndpoint {{
    /// Create a UDS endpoint.
    #[must_use]
    pub fn uds(path: impl Into<String>) -> Self {{
        Self::Uds {{ path: path.into() }}
    }}

    /// Create a TCP endpoint.
    #[must_use]
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {{
        Self::Tcp {{ host: host.into(), port }}
    }}

    /// Parse from the `TRANSPORT_ENDPOINT` env var, with a UDS fallback.
    #[must_use]
    pub fn from_env_or_default(primal_name: &str) -> Self {{
        std::env::var("TRANSPORT_ENDPOINT")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| {{
                let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
                    .unwrap_or_else(|_| "/tmp".to_owned());
                Self::uds(format!("{{runtime_dir}}/biomeos/{{primal_name}}.sock"))
            }})
    }}
}}

impl std::fmt::Display for TransportEndpoint {{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {{
        match self {{
            Self::Uds {{ path }} => write!(f, "unix://{{path}}"),
            Self::Tcp {{ host, port }} => write!(f, "tcp://{{host}}:{{port}}"),
            Self::MeshRelay {{ peer_id, capability }} => {{
                write!(f, "mesh://{{peer_id}}/{{capability}}")
            }}
        }}
    }}
}}

/// Unified async read/write stream across transport types.
pub enum TransportStream {{
    /// Unix domain socket stream.
    Unix(UnixStream),
    /// TCP stream.
    Tcp(TcpStream),
}}

/// Connect to a primal using a resolved endpoint.
///
/// # Errors
///
/// Returns `io::Error` if the connection fails.
pub async fn connect_transport(endpoint: &TransportEndpoint) -> std::io::Result<TransportStream> {{
    match endpoint {{
        TransportEndpoint::Uds {{ path }} => {{
            let stream = UnixStream::connect(path).await?;
            Ok(TransportStream::Unix(stream))
        }}
        TransportEndpoint::Tcp {{ host, port }} => {{
            let stream = TcpStream::connect(format!("{{host}}:{{port}}")).await?;
            Ok(TransportStream::Tcp(stream))
        }}
        TransportEndpoint::MeshRelay {{ .. }} => {{
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "mesh relay requires songbird routing — use ipc.resolve",
            ))
        }}
    }}
}}

impl tokio::io::AsyncRead for TransportStream {{
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {{
        match self.get_mut() {{
            Self::Unix(s) => std::pin::Pin::new(s).poll_read(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }}
    }}
}}

impl tokio::io::AsyncWrite for TransportStream {{
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {{
        match self.get_mut() {{
            Self::Unix(s) => std::pin::Pin::new(s).poll_write(cx, buf),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }}
    }}

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {{
        match self.get_mut() {{
            Self::Unix(s) => std::pin::Pin::new(s).poll_flush(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_flush(cx),
        }}
    }}

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {{
        match self.get_mut() {{
            Self::Unix(s) => std::pin::Pin::new(s).poll_shutdown(cx),
            Self::Tcp(s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }}
    }}
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn uds_roundtrip() {{
        let ep = TransportEndpoint::uds("/run/user/1000/biomeos/{name_lower}.sock");
        let json = serde_json::to_string(&ep).unwrap();
        let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep, back);
    }}

    #[test]
    fn tcp_roundtrip() {{
        let ep = TransportEndpoint::tcp("192.168.1.144", 7700);
        let json = serde_json::to_string(&ep).unwrap();
        let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
        assert_eq!(ep, back);
    }}

    #[test]
    fn wire_compatible_with_ecosystem() {{
        let uds: TransportEndpoint =
            serde_json::from_str(r#"{{"transport":"uds","path":"/tmp/test.sock"}}"#).unwrap();
        assert!(matches!(uds, TransportEndpoint::Uds {{ .. }}));

        let tcp: TransportEndpoint =
            serde_json::from_str(r#"{{"transport":"tcp","host":"10.0.0.1","port":8080}}"#).unwrap();
        assert!(matches!(tcp, TransportEndpoint::Tcp {{ .. }}));

        let relay: TransportEndpoint = serde_json::from_str(
            r#"{{"transport":"mesh_relay","peer_id":"gate","capability":"crypto"}}"#,
        )
        .unwrap();
        assert!(matches!(relay, TransportEndpoint::MeshRelay {{ .. }}));
    }}

    #[test]
    fn from_env_or_default_fallback() {{
        std::env::remove_var("TRANSPORT_ENDPOINT");
        let ep = TransportEndpoint::from_env_or_default("{name_lower}");
        assert!(matches!(ep, TransportEndpoint::Uds {{ .. }}));
        let path = match &ep {{
            TransportEndpoint::Uds {{ path }} => path.as_str(),
            _ => panic!("expected UDS"),
        }};
        assert!(path.contains("{name_lower}"));
        assert!(path.ends_with(".sock"));
    }}
}}
"##
    )
}

/// Convert a primal name to a Rust type name (uppercase first letter).
fn primal_rust_type_name(name: &str) -> String {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_uppercase().collect::<String>() + chars.as_str()
}
