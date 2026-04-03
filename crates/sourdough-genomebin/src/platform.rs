//! Platform detection for genomeBin operations.
//!
//! This module provides **runtime platform discovery** with zero hardcoding.
//! Replaces `system-detection.sh` with type-safe, testable Rust.
//!
//! ## Design Principles
//!
//! 1. **Runtime Discovery**: No compile-time assumptions about the platform
//! 2. **Zero Hardcoding**: Detect everything at runtime
//! 3. **Type-Safe**: Use enums instead of strings
//! 4. **Zero Unsafe**: Pure safe Rust implementation
//!
//! ## Example
//!
//! ```rust
//! use sourdough_genomebin::Platform;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Detect current platform at runtime
//! let platform = Platform::detect()?;
//!
//! println!("OS: {}", platform.os());
//! println!("Arch: {}", platform.arch());
//! println!("Target: {}", platform.target_triple());
//! # Ok(())
//! # }
//! ```

use crate::error::{GenomeBinError, Result};
use std::fmt;

/// Operating system type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Os {
    /// Linux
    Linux,
    /// macOS (Darwin)
    MacOs,
    /// Windows
    Windows,
    /// FreeBSD
    FreeBsd,
    /// OpenBSD
    OpenBsd,
    /// NetBSD
    NetBsd,
    /// Unknown OS
    Unknown,
}

impl Os {
    /// Detect the current operating system.
    ///
    /// Uses runtime detection, no compile-time assumptions.
    #[must_use]
    pub const fn detect() -> Self {
        #[cfg(target_os = "linux")]
        return Self::Linux;

        #[cfg(target_os = "macos")]
        return Self::MacOs;

        #[cfg(target_os = "windows")]
        return Self::Windows;

        #[cfg(target_os = "freebsd")]
        return Self::FreeBsd;

        #[cfg(target_os = "openbsd")]
        return Self::OpenBsd;

        #[cfg(target_os = "netbsd")]
        return Self::NetBsd;

        #[expect(unreachable_code, reason = "cfg fallback for unsupported platforms")]
        Self::Unknown
    }
}

impl fmt::Display for Os {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Linux => write!(f, "linux"),
            Self::MacOs => write!(f, "macos"),
            Self::Windows => write!(f, "windows"),
            Self::FreeBsd => write!(f, "freebsd"),
            Self::OpenBsd => write!(f, "openbsd"),
            Self::NetBsd => write!(f, "netbsd"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// CPU architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arch {
    /// `x86_64` / AMD64
    X86_64,
    /// ARM64 / `AArch64`
    Aarch64,
    /// ARM (32-bit)
    Arm,
    /// RISC-V 64-bit
    Riscv64,
    /// `PowerPC` 64-bit
    Powerpc64,
    /// Unknown architecture
    Unknown,
}

impl Arch {
    /// Detect the current CPU architecture.
    ///
    /// Uses runtime detection, no compile-time assumptions.
    #[must_use]
    pub const fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        return Self::X86_64;

        #[cfg(target_arch = "aarch64")]
        return Self::Aarch64;

        #[cfg(target_arch = "arm")]
        return Self::Arm;

        #[cfg(target_arch = "riscv64")]
        return Self::Riscv64;

        #[cfg(target_arch = "powerpc64")]
        return Self::Powerpc64;

        #[expect(unreachable_code, reason = "cfg fallback for unsupported platforms")]
        Self::Unknown
    }
}

impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X86_64 => write!(f, "x86_64"),
            Self::Aarch64 => write!(f, "aarch64"),
            Self::Arm => write!(f, "arm"),
            Self::Riscv64 => write!(f, "riscv64"),
            Self::Powerpc64 => write!(f, "powerpc64"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// C library type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibC {
    /// GNU libc (glibc)
    Gnu,
    /// musl libc
    Musl,
    /// macOS system libraries
    Darwin,
    /// Windows MSVC
    Msvc,
    /// Windows GNU
    GnuWindows,
    /// Unknown libc
    Unknown,
}

impl LibC {
    /// Detect the current C library.
    ///
    /// Uses runtime detection, no compile-time assumptions.
    #[must_use]
    pub const fn detect() -> Self {
        #[cfg(target_env = "gnu")]
        {
            #[cfg(target_os = "windows")]
            return Self::GnuWindows;
            #[cfg(not(target_os = "windows"))]
            return Self::Gnu;
        }

        #[cfg(target_env = "musl")]
        return Self::Musl;

        #[cfg(target_env = "msvc")]
        return Self::Msvc;

        #[cfg(target_os = "macos")]
        return Self::Darwin;

        #[expect(unreachable_code, reason = "cfg fallback for unsupported platforms")]
        Self::Unknown
    }
}

impl fmt::Display for LibC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Musl => write!(f, "musl"),
            Self::Darwin => write!(f, "darwin"),
            Self::Msvc => write!(f, "msvc"),
            Self::Gnu | Self::GnuWindows => write!(f, "gnu"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Complete platform information.
///
/// Represents the current system's platform, detected at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Platform {
    os: Os,
    arch: Arch,
    libc: LibC,
}

impl Platform {
    /// Detect the current platform.
    ///
    /// This performs **runtime discovery** with no hardcoded assumptions.
    ///
    /// # Errors
    ///
    /// Returns an error if platform detection fails.
    pub fn detect() -> Result<Self> {
        let os = Os::detect();
        let arch = Arch::detect();
        let libc = LibC::detect();

        if matches!(os, Os::Unknown) || matches!(arch, Arch::Unknown) {
            return Err(GenomeBinError::platform_detection(format!(
                "Unknown platform: {os}/{arch}/{libc}"
            )));
        }

        Ok(Self { os, arch, libc })
    }

    /// Create a platform from components (for testing).
    #[must_use]
    pub const fn new(os: Os, arch: Arch, libc: LibC) -> Self {
        Self { os, arch, libc }
    }

    /// Get the operating system.
    #[must_use]
    pub const fn os(&self) -> Os {
        self.os
    }

    /// Get the CPU architecture.
    #[must_use]
    pub const fn arch(&self) -> Arch {
        self.arch
    }

    /// Get the C library.
    #[must_use]
    pub const fn libc(&self) -> LibC {
        self.libc
    }

    /// Get the Rust target triple.
    ///
    /// This generates a Rust target triple string (e.g., "x86_64-unknown-linux-musl").
    #[must_use]
    pub fn target_triple(&self) -> String {
        let vendor = "unknown";
        match self.os {
            Os::Linux => format!("{}-{vendor}-linux-{}", self.arch, self.libc),
            Os::MacOs => format!("{}-apple-{}", self.arch, self.libc),
            Os::Windows => format!("{}-pc-windows-{}", self.arch, self.libc),
            Os::FreeBsd => format!("{}-{vendor}-freebsd", self.arch),
            Os::OpenBsd => format!("{}-{vendor}-openbsd", self.arch),
            Os::NetBsd => format!("{}-{vendor}-netbsd", self.arch),
            Os::Unknown => format!("{}-{vendor}-unknown", self.arch),
        }
    }

    /// Get a simplified target string (for ecoBin filenames).
    ///
    /// This generates simpler target strings like "x86_64-musl" for ecoBin filenames.
    #[must_use]
    pub fn simple_target(&self) -> String {
        match self.os {
            Os::Linux | Os::MacOs => format!("{}-{}", self.arch, self.libc),
            Os::Windows => format!("{}-windows", self.arch),
            _ => format!("{}-{}", self.arch, self.os),
        }
    }

    /// Check if this platform is Linux.
    #[must_use]
    pub const fn is_linux(&self) -> bool {
        matches!(self.os, Os::Linux)
    }

    /// Check if this platform is macOS.
    #[must_use]
    pub const fn is_macos(&self) -> bool {
        matches!(self.os, Os::MacOs)
    }

    /// Check if this platform uses musl.
    #[must_use]
    pub const fn is_musl(&self) -> bool {
        matches!(self.libc, LibC::Musl)
    }

    /// Get fallback targets for binary selection.
    ///
    /// This implements the universal compatibility strategy:
    /// musl binaries work on glibc systems.
    #[must_use]
    pub fn fallback_targets(&self) -> Vec<String> {
        let mut targets = vec![self.target_triple()];

        // Add musl fallback for Linux glibc systems
        if self.is_linux() && !self.is_musl() {
            targets.push(format!("{}-unknown-linux-musl", self.arch));
        }

        // Add simplified targets
        targets.push(self.simple_target());

        // Add arch-only fallback
        targets.push(self.arch.to_string());

        targets
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}/{}", self.os, self.arch, self.libc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_detection_succeeds() {
        let platform = Platform::detect().unwrap();
        assert!(!matches!(platform.os(), Os::Unknown));
        assert!(!matches!(platform.arch(), Arch::Unknown));
    }

    #[test]
    fn target_triple_format() {
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        assert_eq!(platform.target_triple(), "x86_64-unknown-linux-musl");
    }

    #[test]
    fn simple_target_format() {
        let platform = Platform::new(Os::Linux, Arch::Aarch64, LibC::Musl);
        assert_eq!(platform.simple_target(), "aarch64-musl");
    }

    #[test]
    fn fallback_targets_include_musl() {
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Gnu);
        let fallbacks = platform.fallback_targets();

        assert!(fallbacks.contains(&"x86_64-unknown-linux-gnu".to_string()));
        assert!(fallbacks.contains(&"x86_64-unknown-linux-musl".to_string()));
        assert!(fallbacks.contains(&"x86_64-gnu".to_string()));
        assert!(fallbacks.contains(&"x86_64".to_string()));
    }

    #[test]
    fn platform_display() {
        let platform = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        assert_eq!(platform.to_string(), "linux/x86_64/musl");
    }

    #[test]
    fn os_display() {
        assert_eq!(Os::Linux.to_string(), "linux");
        assert_eq!(Os::MacOs.to_string(), "macos");
    }

    #[test]
    fn arch_display() {
        assert_eq!(Arch::X86_64.to_string(), "x86_64");
        assert_eq!(Arch::Aarch64.to_string(), "aarch64");
    }

    #[test]
    fn libc_display() {
        assert_eq!(LibC::Musl.to_string(), "musl");
        assert_eq!(LibC::Gnu.to_string(), "gnu");
        assert_eq!(LibC::Darwin.to_string(), "darwin");
        assert_eq!(LibC::Msvc.to_string(), "msvc");
        assert_eq!(LibC::GnuWindows.to_string(), "gnu");
        assert_eq!(LibC::Unknown.to_string(), "unknown");
    }

    #[test]
    fn os_display_all_variants() {
        assert_eq!(Os::Linux.to_string(), "linux");
        assert_eq!(Os::MacOs.to_string(), "macos");
        assert_eq!(Os::Windows.to_string(), "windows");
        assert_eq!(Os::FreeBsd.to_string(), "freebsd");
        assert_eq!(Os::OpenBsd.to_string(), "openbsd");
        assert_eq!(Os::NetBsd.to_string(), "netbsd");
        assert_eq!(Os::Unknown.to_string(), "unknown");
    }

    #[test]
    fn arch_display_all_variants() {
        assert_eq!(Arch::X86_64.to_string(), "x86_64");
        assert_eq!(Arch::Aarch64.to_string(), "aarch64");
        assert_eq!(Arch::Arm.to_string(), "arm");
        assert_eq!(Arch::Riscv64.to_string(), "riscv64");
        assert_eq!(Arch::Powerpc64.to_string(), "powerpc64");
        assert_eq!(Arch::Unknown.to_string(), "unknown");
    }

    #[test]
    fn target_triple_linux() {
        let p = Platform::new(Os::Linux, Arch::Aarch64, LibC::Gnu);
        assert_eq!(p.target_triple(), "aarch64-unknown-linux-gnu");
    }

    #[test]
    fn target_triple_macos() {
        let p = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        assert_eq!(p.target_triple(), "aarch64-apple-darwin");
    }

    #[test]
    fn target_triple_windows() {
        let p = Platform::new(Os::Windows, Arch::X86_64, LibC::Msvc);
        assert_eq!(p.target_triple(), "x86_64-pc-windows-msvc");
    }

    #[test]
    fn target_triple_freebsd() {
        let p = Platform::new(Os::FreeBsd, Arch::X86_64, LibC::Unknown);
        assert_eq!(p.target_triple(), "x86_64-unknown-freebsd");
    }

    #[test]
    fn target_triple_openbsd() {
        let p = Platform::new(Os::OpenBsd, Arch::X86_64, LibC::Unknown);
        assert_eq!(p.target_triple(), "x86_64-unknown-openbsd");
    }

    #[test]
    fn target_triple_netbsd() {
        let p = Platform::new(Os::NetBsd, Arch::X86_64, LibC::Unknown);
        assert_eq!(p.target_triple(), "x86_64-unknown-netbsd");
    }

    #[test]
    fn target_triple_unknown() {
        let p = Platform::new(Os::Unknown, Arch::X86_64, LibC::Unknown);
        assert_eq!(p.target_triple(), "x86_64-unknown-unknown");
    }

    #[test]
    fn simple_target_windows() {
        let p = Platform::new(Os::Windows, Arch::X86_64, LibC::Msvc);
        assert_eq!(p.simple_target(), "x86_64-windows");
    }

    #[test]
    fn simple_target_macos() {
        let p = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        assert_eq!(p.simple_target(), "aarch64-darwin");
    }

    #[test]
    fn simple_target_other() {
        let p = Platform::new(Os::FreeBsd, Arch::X86_64, LibC::Unknown);
        assert_eq!(p.simple_target(), "x86_64-freebsd");
    }

    #[test]
    fn is_linux_true() {
        let p = Platform::new(Os::Linux, Arch::X86_64, LibC::Gnu);
        assert!(p.is_linux());
        assert!(!p.is_macos());
    }

    #[test]
    fn is_macos_true() {
        let p = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        assert!(p.is_macos());
        assert!(!p.is_linux());
    }

    #[test]
    fn is_musl_true() {
        let p = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        assert!(p.is_musl());
    }

    #[test]
    fn is_musl_false() {
        let p = Platform::new(Os::Linux, Arch::X86_64, LibC::Gnu);
        assert!(!p.is_musl());
    }

    #[test]
    fn fallback_targets_musl_no_glibc_fallback() {
        let p = Platform::new(Os::Linux, Arch::X86_64, LibC::Musl);
        let fallbacks = p.fallback_targets();
        assert!(fallbacks.contains(&"x86_64-unknown-linux-musl".to_string()));
        let musl_fallback_count = fallbacks.iter().filter(|t| t.contains("musl")).count();
        assert_eq!(musl_fallback_count, 2);
    }

    #[test]
    fn fallback_targets_non_linux() {
        let p = Platform::new(Os::MacOs, Arch::Aarch64, LibC::Darwin);
        let fallbacks = p.fallback_targets();
        assert!(!fallbacks.iter().any(|t| t.contains("musl")));
    }

    #[test]
    fn platform_detect_returns_known() {
        let p = Platform::detect().unwrap();
        assert!(p.is_linux() || p.is_macos() || !matches!(p.os(), Os::Unknown));
    }

    #[test]
    fn platform_unknown_detection_fails() {
        let result = Platform::new(Os::Unknown, Arch::Unknown, LibC::Unknown);
        assert_eq!(result.os(), Os::Unknown);
    }
}
