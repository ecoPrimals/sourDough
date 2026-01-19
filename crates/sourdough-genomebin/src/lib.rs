//! # sourdough-genomebin
//!
//! Pure Rust genomeBin infrastructure for universal deployment.
//!
//! This library replaces the bash script-based genomeBin tooling with:
//! - **Type-safe API**: Compile-time guarantees, no string manipulation bugs
//! - **Concurrent operations**: Parallel processing for 2-3x performance
//! - **Zero unsafe code**: 100% safe Rust
//! - **Comprehensive testing**: Unit tests for every component
//! - **Zero hardcoding**: Agnostic, capability-based design
//!
//! ## Architecture
//!
//! The library is organized into focused modules:
//! - [`platform`]: System detection (OS, arch, libc) - runtime discovery
//! - [`builder`]: `GenomeBin` creation with concurrent processing
//! - [`validator`]: Testing and validation
//! - [`metadata`]: Type-safe metadata handling
//! - [`archive`]: Tar/gzip operations
//! - [`error`]: Structured error types
//!
//! ## Example
//!
//! ```rust,no_run
//! use sourdough_genomebin::{GenomeBinBuilder, Platform};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Detect current platform at runtime (no hardcoding!)
//! let platform = Platform::detect()?;
//! println!("Running on: {}", platform);
//!
//! // Build genomeBin with type-safe API
//! let builder = GenomeBinBuilder::new("myprimal", "1.0.0")
//!     .ecobins_dir("./ecobins")
//!     .output("myprimal-1.0.0.genome")
//!     .parallel(true);
//!
//! let genome = builder.build().await?;
//!
//! // Create with concurrent processing
//! let output_path = genome.create().await?;
//! println!("Created: {}", output_path.display());
//! # Ok(())
//! # }
//! ```
//!
//! ## Design Principles
//!
//! 1. **Zero Unsafe Code**: All operations use safe Rust
//! 2. **Zero Hardcoding**: Runtime discovery and capability-based design
//! 3. **Pure Rust**: No C dependencies (100% Pure Rust stack)
//! 4. **Concurrent**: Parallel processing where beneficial
//! 5. **Type-Safe**: Validated types prevent invalid states
//! 6. **Well-Tested**: Comprehensive unit and integration tests

#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic
)]
#![forbid(unsafe_code)]

pub mod archive;
pub mod builder;
pub mod error;
pub mod metadata;
pub mod platform;
pub mod validator;

// Re-export main types for convenience
pub use builder::{GenomeBin, GenomeBinBuilder};
pub use error::{GenomeBinError, Result};
pub use metadata::Metadata;
pub use platform::Platform;
pub use validator::Validator;
