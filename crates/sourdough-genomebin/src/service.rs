//! Cross-platform service template generation.
//!
//! Generates platform-native service management files from a unified
//! [`ServiceConfig`] specification:
//!
//! - **Linux**: systemd unit files (`.service`)
//! - **macOS**: launchd property lists (`.plist`)
//! - **Windows**: NSSM/sc wrapper scripts (future)
//!
//! ## Design
//!
//! Uses [`Platform`] detection to select the correct template at runtime.
//! All templates are generated as `String` — no filesystem writes happen here
//! (caller decides where to put them).
//!
//! ## Example
//!
//! ```rust
//! use sourdough_genomebin::service::{ServiceConfig, ServiceTemplate};
//! use sourdough_genomebin::Platform;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ServiceConfig::new("beardog", "/usr/local/bin/beardog")
//!     .description("Trust — crypto, BTSP, FIDO2, Ed25519 signing")
//!     .user("ecoprimals")
//!     .data_dir("/var/lib/ecoprimals/beardog")
//!     .log_dir("/var/log/ecoprimals");
//!
//! let platform = Platform::detect()?;
//! let template = ServiceTemplate::generate(&config, &platform);
//! println!("{}", template.content());
//! # Ok(())
//! # }
//! ```

use crate::platform::{Os, Platform};

/// Service configuration — platform-agnostic specification.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    /// Primal binary name (e.g., "beardog").
    pub name: String,
    /// Absolute path to the binary executable.
    pub binary_path: String,
    /// Human-readable service description.
    pub description: String,
    /// Command-line arguments passed to the binary.
    pub args: Vec<String>,
    /// Unix user to run the service as.
    pub user: String,
    /// Unix group to run the service as.
    pub group: String,
    /// Data directory for runtime state.
    pub data_dir: String,
    /// Configuration directory (config.toml, environment file).
    pub config_dir: String,
    /// Log output directory.
    pub log_dir: String,
    /// RUST_LOG environment variable value.
    pub rust_log: String,
    /// Max open file descriptors (NOFILE limit).
    pub fd_limit: u64,
    /// Seconds to wait before restarting on failure.
    pub restart_delay_secs: u64,
}

impl ServiceConfig {
    /// Create a new service config with sensible defaults.
    #[must_use]
    pub fn new(name: &str, binary_path: &str) -> Self {
        let name_str = name.to_string();
        Self {
            name: name_str.clone(),
            binary_path: binary_path.to_string(),
            description: format!("{name} - ecoPrimals primal"),
            args: Vec::new(),
            user: "ecoprimals".to_string(),
            group: "ecoprimals".to_string(),
            data_dir: format!("/var/lib/ecoprimals/{name_str}"),
            config_dir: format!("/etc/ecoprimals/{name_str}"),
            log_dir: "/var/log/ecoprimals".to_string(),
            rust_log: format!("{name_str}=info"),
            fd_limit: 65536,
            restart_delay_secs: 5,
        }
    }

    /// Set the service description.
    #[must_use]
    pub fn description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Set the service user.
    #[must_use]
    pub fn user(mut self, user: &str) -> Self {
        self.user = user.to_string();
        self
    }

    /// Set the service group.
    #[must_use]
    pub fn group(mut self, group: &str) -> Self {
        self.group = group.to_string();
        self
    }

    /// Set the data directory.
    #[must_use]
    pub fn data_dir(mut self, dir: &str) -> Self {
        self.data_dir = dir.to_string();
        self
    }

    /// Set the config directory.
    #[must_use]
    pub fn config_dir(mut self, dir: &str) -> Self {
        self.config_dir = dir.to_string();
        self
    }

    /// Set the log directory.
    #[must_use]
    pub fn log_dir(mut self, dir: &str) -> Self {
        self.log_dir = dir.to_string();
        self
    }

    /// Add command-line arguments.
    #[must_use]
    pub fn args(mut self, args: &[&str]) -> Self {
        self.args = args.iter().map(|s| (*s).to_string()).collect();
        self
    }

    /// Set RUST_LOG level.
    #[must_use]
    pub fn rust_log(mut self, level: &str) -> Self {
        self.rust_log = level.to_string();
        self
    }
}

/// Generated service template with metadata.
#[derive(Debug, Clone)]
pub struct ServiceTemplate {
    content: String,
    filename: String,
    install_path: String,
}

impl ServiceTemplate {
    /// Generate a platform-native service template.
    #[must_use]
    pub fn generate(config: &ServiceConfig, platform: &Platform) -> Self {
        match platform.os() {
            Os::Linux | Os::Android => Self::systemd(config),
            Os::MacOs => Self::launchd(config),
            _ => Self::systemd(config),
        }
    }

    /// Get the generated template content.
    #[must_use]
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Get the recommended filename.
    #[must_use]
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// Get the recommended install path.
    #[must_use]
    pub fn install_path(&self) -> &str {
        &self.install_path
    }

    fn systemd(config: &ServiceConfig) -> Self {
        let args_str = config.args.join(" ");
        let name_upper = config.name.to_uppercase().replace('-', "_");

        let content = format!(
            r#"[Unit]
Description={description}
Documentation=https://docs.primals.eco/{name}/
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User={user}
Group={group}
ExecStart={binary} {args}
ExecReload=/bin/kill -HUP $MAINPID
Restart=on-failure
RestartSec={restart_delay}s
StandardOutput=journal
StandardError=journal
SyslogIdentifier={name}

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths={data_dir} {config_dir}
ProtectKernelTunables=true
ProtectKernelModules=true
ProtectControlGroups=true

# Resource limits
LimitNOFILE={fd_limit}
LimitNPROC=512

# Environment
Environment="RUST_LOG={rust_log}"
Environment="{name_upper}_CONFIG={config_dir}/config.toml"
EnvironmentFile=-{config_dir}/environment

[Install]
WantedBy=multi-user.target
"#,
            description = config.description,
            name = config.name,
            user = config.user,
            group = config.group,
            binary = config.binary_path,
            args = args_str,
            restart_delay = config.restart_delay_secs,
            data_dir = config.data_dir,
            config_dir = config.config_dir,
            fd_limit = config.fd_limit,
            rust_log = config.rust_log,
            name_upper = name_upper,
        );

        Self {
            content,
            filename: format!("eco-{}.service", config.name),
            install_path: format!("/etc/systemd/system/eco-{}.service", config.name),
        }
    }

    fn launchd(config: &ServiceConfig) -> Self {
        let args_xml: String = config
            .args
            .iter()
            .map(|a| format!("        <string>{a}</string>"))
            .collect::<Vec<_>>()
            .join("\n");

        let args_section = if args_xml.is_empty() {
            format!(
                "    <key>ProgramArguments</key>\n    <array>\n        <string>{}</string>\n    </array>",
                config.binary_path
            )
        } else {
            format!(
                "    <key>ProgramArguments</key>\n    <array>\n        <string>{}</string>\n{}\n    </array>",
                config.binary_path, args_xml
            )
        };

        let name_upper = config.name.to_uppercase().replace('-', "_");

        let content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>eco.primals.{name}</string>

{args_section}

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>ThrottleInterval</key>
    <integer>{restart_delay}</integer>

    <key>StandardOutPath</key>
    <string>{log_dir}/{name}.log</string>

    <key>StandardErrorPath</key>
    <string>{log_dir}/{name}.error.log</string>

    <key>WorkingDirectory</key>
    <string>{data_dir}</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>{rust_log}</string>
        <key>{name_upper}_CONFIG</key>
        <string>{config_dir}/config.toml</string>
    </dict>

    <key>UserName</key>
    <string>{user}</string>

    <key>GroupName</key>
    <string>{group}</string>

    <key>SoftResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>{fd_limit}</integer>
    </dict>

    <key>HardResourceLimits</key>
    <dict>
        <key>NumberOfFiles</key>
        <integer>{fd_limit}</integer>
    </dict>

    <key>ProcessType</key>
    <string>Standard</string>

    <key>Nice</key>
    <integer>0</integer>
</dict>
</plist>
"#,
            name = config.name,
            args_section = args_section,
            restart_delay = config.restart_delay_secs,
            log_dir = config.log_dir,
            data_dir = config.data_dir,
            rust_log = config.rust_log,
            name_upper = name_upper,
            config_dir = config.config_dir,
            user = config.user,
            group = config.group,
            fd_limit = config.fd_limit,
        );

        Self {
            content,
            filename: format!("eco.primals.{}.plist", config.name),
            install_path: format!(
                "/Library/LaunchDaemons/eco.primals.{}.plist",
                config.name
            ),
        }
    }
}

/// Generate service templates for all Tower Atomic primals.
///
/// Tower Atomic = bearDog + songBird + skunkBat + swarmVine (shared
/// electron cloud — present in ALL compositions via bonding model).
#[must_use]
pub fn tower_atomic_templates(platform: &Platform, bin_dir: &str) -> Vec<ServiceTemplate> {
    let primals = [
        ("beardog", "Trust — crypto, BTSP, FIDO2, Ed25519 signing"),
        ("songbird", "Discovery — mesh, IPC, relay, drawbridge"),
        ("skunkbat", "Defense — anomaly detection, protocol audit"),
        ("swarmvine", "Gossip — epidemic protocol, ant colony, cascade"),
    ];

    primals
        .iter()
        .map(|(name, desc)| {
            let config = ServiceConfig::new(name, &format!("{bin_dir}/{name}"))
                .description(desc);
            ServiceTemplate::generate(&config, platform)
        })
        .collect()
}

/// Generate service templates for all NUCLEUS primals.
///
/// NUCLEUS = Tower (bearDog + songBird + skunkBat + swarmVine) +
/// Nest (nestGate + rhizoCrypt + loamSpine + sweetGrass) +
/// Node (toadStool + barraCuda + coralReef) +
/// biomeOS + petalTongue + squirrel + cellMembrane.
#[must_use]
pub fn nucleus_templates(platform: &Platform, bin_dir: &str) -> Vec<ServiceTemplate> {
    let primals = [
        ("beardog", "Trust — crypto, BTSP, FIDO2, Ed25519 signing"),
        ("songbird", "Discovery — mesh, IPC, relay, drawbridge"),
        ("skunkbat", "Defense — anomaly detection, protocol audit"),
        ("swarmvine", "Gossip — epidemic protocol, ant colony, cascade"),
        ("nestgate", "Content-addressed storage — CAS, provenance"),
        ("rhizocrypt", "Lineage DAG — content identity, federation"),
        ("loamspine", "Certificate ledger — lifecycle, verification"),
        ("sweetgrass", "Attribution braids — provenance chains"),
        ("toadstool", "Compute dispatch — GPU, wgpu, hardware learning"),
        ("barracuda", "Tensor math — linear algebra, GPU compute"),
        ("coralreef", "Shader compilation — WGSL, SPIR-V, PTX"),
        ("biomeos", "Orchestrator — Neural API, signal graphs, NUCLEUS"),
        ("squirrel", "AI assistant — MCP, ML"),
        ("petaltongue", "Visualization — WASM, WebGL, rendering"),
        ("cellmembrane", "Sovereignty — topology, depot, cascade validation"),
        ("sourdough", "Factory — standards validator"),
    ];

    primals
        .iter()
        .map(|(name, desc)| {
            let config = ServiceConfig::new(name, &format!("{bin_dir}/{name}"))
                .description(desc);
            ServiceTemplate::generate(&config, platform)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{Arch, LibC};

    #[test]
    fn systemd_template_generation() {
        let config = ServiceConfig::new("beardog", "/usr/local/bin/beardog")
            .description("Trust primal");
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        let template = ServiceTemplate::generate(&config, &platform);

        assert_eq!(template.filename(), "eco-beardog.service");
        assert!(template.content().contains("[Unit]"));
        assert!(template.content().contains("Trust primal"));
        assert!(template.content().contains("/usr/local/bin/beardog"));
        assert!(template.content().contains("BEARDOG_CONFIG"));
        assert!(template.content().contains("LimitNOFILE=65536"));
        assert!(template.content().contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn launchd_template_generation() {
        let config = ServiceConfig::new("beardog", "/usr/local/bin/beardog")
            .description("Trust primal");
        let platform = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        let template = ServiceTemplate::generate(&config, &platform);

        assert_eq!(template.filename(), "eco.primals.beardog.plist");
        assert!(template.content().contains("eco.primals.beardog"));
        assert!(template.content().contains("/usr/local/bin/beardog"));
        assert!(template.content().contains("BEARDOG_CONFIG"));
        assert!(template.content().contains("NumberOfFiles"));
        assert!(template.content().contains("<!DOCTYPE plist"));
    }

    #[test]
    fn launchd_install_path() {
        let config = ServiceConfig::new("songbird", "/opt/ecoprimals/bin/songbird");
        let platform = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        let template = ServiceTemplate::generate(&config, &platform);

        assert_eq!(
            template.install_path(),
            "/Library/LaunchDaemons/eco.primals.songbird.plist"
        );
    }

    #[test]
    fn systemd_install_path() {
        let config = ServiceConfig::new("songbird", "/usr/local/bin/songbird");
        let platform = Platform::new(Os::Linux, Arch::Aarch64, LibC::Gnu);
        let template = ServiceTemplate::generate(&config, &platform);

        assert_eq!(
            template.install_path(),
            "/etc/systemd/system/eco-songbird.service"
        );
    }

    #[test]
    fn config_builder_chain() {
        let config = ServiceConfig::new("nestgate", "/usr/local/bin/nestgate")
            .description("CAS primal")
            .user("nest")
            .group("storage")
            .data_dir("/srv/nestgate")
            .config_dir("/etc/nestgate")
            .log_dir("/var/log/nestgate")
            .args(&["--port", "7700"])
            .rust_log("nestgate=debug");

        assert_eq!(config.name, "nestgate");
        assert_eq!(config.user, "nest");
        assert_eq!(config.group, "storage");
        assert_eq!(config.data_dir, "/srv/nestgate");
        assert_eq!(config.args, vec!["--port", "7700"]);
    }

    #[test]
    fn tower_atomic_generates_four() {
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        let templates = tower_atomic_templates(&platform, "/usr/local/bin");
        assert_eq!(templates.len(), 4);
        let names: Vec<_> = templates.iter().map(|t| t.filename()).collect();
        assert!(names.contains(&"eco-swarmvine.service"));
    }

    #[test]
    fn nucleus_generates_sixteen() {
        let platform = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        let templates = nucleus_templates(&platform, "/usr/local/bin");
        assert_eq!(templates.len(), 16);
        assert!(templates.iter().all(|t| t.filename().ends_with(".plist")));
        let names: Vec<_> = templates.iter().map(|t| t.filename()).collect();
        assert!(names.contains(&"eco.primals.cellmembrane.plist"));
    }

    #[test]
    fn systemd_with_args() {
        let config = ServiceConfig::new("biomeos", "/usr/local/bin/biomeos")
            .args(&["--composition", "nucleus", "--port", "9000"]);
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        let template = ServiceTemplate::generate(&config, &platform);

        assert!(template.content().contains("--composition nucleus --port 9000"));
    }

    #[test]
    fn launchd_with_args() {
        let config = ServiceConfig::new("biomeos", "/usr/local/bin/biomeos")
            .args(&["--composition", "nucleus"]);
        let platform = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        let template = ServiceTemplate::generate(&config, &platform);

        assert!(template.content().contains("<string>--composition</string>"));
        assert!(template.content().contains("<string>nucleus</string>"));
    }

    #[test]
    fn current_platform_generates() {
        let platform = Platform::detect().unwrap();
        let config = ServiceConfig::new("beardog", "/usr/local/bin/beardog");
        let template = ServiceTemplate::generate(&config, &platform);
        assert!(!template.content().is_empty());
        assert!(!template.filename().is_empty());
    }
}
