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
     \x20\x20\x20\x20\x20 - run: cargo fmt --all -- --check\n\
     \x20\x20\x20\x20\x20 - run: cargo clippy --workspace --all-targets -- -D warnings\n\
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
deny = [
    { crate = "openssl-sys", wrappers = [] },
    { crate = "openssl-src", wrappers = [] },
    { crate = "native-tls", wrappers = [] },
    { crate = "aws-lc-sys", wrappers = [] },
    { crate = "cmake", wrappers = [] },
    { crate = "cc", wrappers = [] },
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
]
skip = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
"#;
