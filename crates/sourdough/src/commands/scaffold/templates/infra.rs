//! Infrastructure templates: CI workflows and cargo-deny configuration.
//!
//! These files wire scaffolded primals into the ecosystem CI/CD pipeline
//! and supply chain auditing from first push.

/// Generate `ci.yml` GitHub Actions workflow.
pub(in crate::commands::scaffold) fn ci_yml(_name: &str) -> String {
    "name: CI\n\
     on: [push, pull_request]\n\
     concurrency:\n\
     \x20 group: ci-${{ github.ref }}\n\
     \x20 cancel-in-progress: true\n\
     jobs:\n\
     \x20 check:\n\
     \x20\x20\x20 runs-on: ubuntu-latest\n\
     \x20\x20\x20 timeout-minutes: 20\n\
     \x20\x20\x20 steps:\n\
     \x20\x20\x20\x20\x20 - uses: actions/checkout@v4\n\
     \x20\x20\x20\x20\x20 - uses: dtolnay/rust-toolchain@stable\n\
     \x20\x20\x20\x20\x20 - uses: Swatinem/rust-cache@v2\n\
     \x20\x20\x20\x20\x20 - run: cargo install --locked cargo-deny\n\
     \x20\x20\x20\x20\x20 - run: cargo fmt --all -- --check\n\
     \x20\x20\x20\x20\x20 - run: cargo clippy --workspace --all-targets -- -D warnings\n\
     \x20\x20\x20\x20\x20 - run: cargo deny check\n\
     \x20\x20\x20\x20\x20 - run: cargo test --workspace\n"
        .to_owned()
}

/// Generate `notify-plasmidbin.yml` workflow.
pub(in crate::commands::scaffold) const NOTIFY_PLASMIDBIN_YML: &str = r#"name: Notify plasmidBin
on:
  push:
    branches: [main]
jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - name: Dispatch rebuild to plasmidBin
        uses: peter-evans/repository-dispatch@v3
        with:
          token: ${{ secrets.PLASMIDBIN_DISPATCH_TOKEN }}
          repository: ecoPrimals/plasmidBin
          event-type: primal-updated
          client-payload: '{"primal": "${{ github.event.repository.name }}", "sha": "${{ github.sha }}"}'
"#;

/// Generate `release.yml` GitHub Actions workflow for musl cross-compilation.
///
/// Produces Tier 1 genomeBin artifacts (`x86_64`, `aarch64`, `armv7` musl-static)
/// on tag push, with BLAKE3 checksums and GitHub Release publishing.
pub(in crate::commands::scaffold) fn release_yml(name: &str) -> String {
    let name_lower = name.to_lowercase();
    format!(
        r#"name: Release

on:
  push:
    tags: ["v*"]

permissions:
  contents: write

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    strategy:
      matrix:
        include:
          - target: x86_64-unknown-linux-musl
            arch: x86_64
            packages: musl-tools
          - target: aarch64-unknown-linux-musl
            arch: aarch64
            packages: musl-tools gcc-aarch64-linux-gnu
            linker_env: CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
          - target: armv7-unknown-linux-musleabihf
            arch: armv7
            packages: musl-tools gcc-arm-linux-gnueabihf
            linker_env: CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=arm-linux-gnueabihf-gcc
          - target: aarch64-linux-android
            arch: aarch64-android
            ndk: true
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{{{ matrix.target }}}}
      - uses: Swatinem/rust-cache@v2
        with:
          key: ${{{{ matrix.target }}}}
      - name: Install cross-compilation tools
        if: matrix.packages
        run: sudo apt-get update && sudo apt-get install -y ${{{{ matrix.packages }}}}
      - name: Setup Android NDK
        if: matrix.ndk
        uses: nttld/setup-ndk@v1
        with:
          ndk-version: r27
      - name: Build release binary
        run: |
          if [ -n "${{{{ matrix.linker_env }}}}" ]; then
            export ${{{{ matrix.linker_env }}}}
          fi
          cargo build --release --target ${{{{ matrix.target }}}} -p {name_lower}-server
      - name: Prepare artifact
        run: |
          mkdir -p dist
          cp target/${{{{ matrix.target }}}}/release/{name_lower} dist/{name_lower}-${{{{ matrix.arch }}}}
      - name: Compute BLAKE3 checksum
        run: |
          cargo install --locked b3sum 2>/dev/null || true
          b3sum dist/{name_lower}-${{{{ matrix.arch }}}} > dist/{name_lower}-${{{{ matrix.arch }}}}.b3
      - uses: actions/upload-artifact@v4
        with:
          name: {name_lower}-${{{{ matrix.arch }}}}
          path: dist/

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true
      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: artifacts/*
          generate_release_notes: true
"#,
    )
}

/// Generate a hardened systemd `.service` unit for a primal deployment.
///
/// Follows the ecosystem membrane pattern: `Type=simple`, restart policy,
/// security hardening (`NoNewPrivileges`, `ProtectSystem=strict`),
/// resource limits, and UDS socket under `/run/biomeos/`.
pub(in crate::commands::scaffold) fn systemd_service(name: &str, role: &str) -> String {
    let name_lower = name.to_lowercase();
    let upper = name_lower.to_uppercase();
    format!(
        "# SPDX-License-Identifier: AGPL-3.0-or-later\n\
         #\n\
         # {name} — {role} deployment\n\
         #\n\
         # Install:\n\
         #   cp {name_lower}-{role}.service /etc/systemd/system/\n\
         #   systemctl daemon-reload\n\
         #   systemctl enable --now {name_lower}-{role}\n\
         \n\
         [Unit]\n\
         Description={name} ({role})\n\
         Documentation=https://primals.eco\n\
         After=network-online.target\n\
         Wants=network-online.target\n\
         StartLimitIntervalSec=60\n\
         StartLimitBurst=5\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart=/opt/{role}/{name_lower}\n\
         \n\
         Environment=TRANSPORT_ENDPOINT={{\"transport\":\"uds\",\"path\":\"/run/biomeos/{name_lower}.sock\"}}\n\
         Environment={upper}_LOG_LEVEL=info\n\
         Environment={upper}_ROLE={role}\n\
         EnvironmentFile=-/opt/{role}/{name_lower}.env\n\
         \n\
         Restart=always\n\
         RestartSec=5\n\
         \n\
         # Security hardening\n\
         NoNewPrivileges=true\n\
         PrivateTmp=true\n\
         ProtectSystem=strict\n\
         ProtectHome=true\n\
         ReadWritePaths=/run/biomeos\n\
         ReadOnlyPaths=/opt/{role}\n\
         \n\
         # Resource limits\n\
         MemoryMax=128M\n\
         CPUQuota=50%\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n"
    )
}

/// Generate `deny.toml` for a scaffolded primal.
pub(in crate::commands::scaffold) const DENY_TOML: &str = r#"# cargo-deny configuration
# https://embarkstudios.github.io/cargo-deny/

[graph]
targets = []
all-features = true

[advisories]
yanked = "deny"
ignore = []

[licenses]
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "MPL-2.0",
    "Unicode-3.0",
    "Zlib",
    "BSL-1.0",
    "AGPL-3.0-or-later",
    "CC0-1.0",
]
confidence-threshold = 0.8
exceptions = []

[licenses.private]
ignore = true

[bans]
multiple-versions = "warn"
wildcards = "warn"
highlight = "all"
allow-wildcard-paths = true

# ecoBin v3.0: C-backed crates banned from application builds.
# blake3 uses cc for optional SIMD backends; use features = ["pure"] if possible,
# but cc is allowed as a wrapper for blake3 per ecosystem convergence standard.
deny = [
    { crate = "openssl-sys", wrappers = [] },
    { crate = "openssl-src", wrappers = [] },
    { crate = "native-tls", wrappers = [] },
    { crate = "aws-lc-sys", wrappers = [] },
    { crate = "cmake", wrappers = [] },
    { crate = "cc", wrappers = ["blake3", "iana-time-zone-haiku"] },
    { crate = "bindgen", wrappers = [] },
    { crate = "bzip2-sys", wrappers = [] },
    { crate = "curl-sys", wrappers = [] },
    { crate = "libz-sys", wrappers = [] },
    { crate = "pkg-config", wrappers = [] },
    { crate = "vcpkg", wrappers = [] },
    { crate = "zstd-sys", wrappers = [] },
    { crate = "lz4-sys", wrappers = [] },
    { crate = "libsqlite3-sys", wrappers = [] },
    { crate = "cryptoki-sys", wrappers = [] },
    { crate = "ring", wrappers = [], reason = "C crypto — ecosystem uses RustCrypto" },
]
skip = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
"#;
