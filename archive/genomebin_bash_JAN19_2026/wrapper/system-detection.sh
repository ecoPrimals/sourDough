#!/usr/bin/env bash
# genomeBin System Detection (STANDARD)
# Detects OS, architecture, init system, and privilege level
# Part of the ecoPrimals genomeBin standard

set -euo pipefail

# Color output
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED='' GREEN='' YELLOW='' BLUE='' NC=''
fi

# Detection functions
detect_os() {
    local os_name
    case "$(uname -s)" in
        Linux*)     os_name="linux";;
        Darwin*)    os_name="macos";;
        FreeBSD*)   os_name="freebsd";;
        OpenBSD*)   os_name="openbsd";;
        NetBSD*)    os_name="netbsd";;
        CYGWIN*|MINGW*|MSYS*) os_name="windows";;
        *)          os_name="unknown";;
    esac
    echo "$os_name"
}

detect_arch() {
    local arch_name
    case "$(uname -m)" in
        x86_64|amd64)   arch_name="x86_64";;
        aarch64|arm64)  arch_name="aarch64";;
        armv7l)         arch_name="armv7";;
        riscv64)        arch_name="riscv64";;
        ppc64le)        arch_name="ppc64le";;
        s390x)          arch_name="s390x";;
        *)              arch_name="unknown";;
    esac
    echo "$arch_name"
}

detect_libc() {
    if ldd --version 2>&1 | grep -iq musl; then
        echo "musl"
    elif ldd --version 2>&1 | grep -iq glibc; then
        echo "gnu"
    elif [[ "$(uname -s)" == "Darwin" ]]; then
        echo "darwin"
    else
        echo "unknown"
    fi
}

detect_init_system() {
    if [[ -d /run/systemd/system ]] || command -v systemctl &>/dev/null; then
        echo "systemd"
    elif [[ -d /System/Library/LaunchDaemons ]] || command -v launchctl &>/dev/null; then
        echo "launchd"
    elif [[ -d /etc/rc.d ]] && [[ "$(uname -s)" == *BSD* ]]; then
        echo "rc.d"
    elif command -v rc-service &>/dev/null; then
        echo "openrc"
    elif [[ -d /etc/init.d ]]; then
        echo "sysvinit"
    else
        echo "unknown"
    fi
}

detect_privilege() {
    if [[ $EUID -eq 0 ]]; then
        echo "root"
    else
        echo "user"
    fi
}

detect_package_manager() {
    if command -v apt-get &>/dev/null; then
        echo "apt"
    elif command -v dnf &>/dev/null; then
        echo "dnf"
    elif command -v yum &>/dev/null; then
        echo "yum"
    elif command -v pacman &>/dev/null; then
        echo "pacman"
    elif command -v apk &>/dev/null; then
        echo "apk"
    elif command -v brew &>/dev/null; then
        echo "brew"
    elif command -v pkg &>/dev/null; then
        echo "pkg"
    else
        echo "unknown"
    fi
}

# Get Linux distribution info
get_linux_distro() {
    if [[ -f /etc/os-release ]]; then
        # shellcheck disable=SC1091
        . /etc/os-release
        echo "${ID:-unknown}"
    elif [[ -f /etc/lsb-release ]]; then
        # shellcheck disable=SC1091
        . /etc/lsb-release
        echo "${DISTRIB_ID:-unknown}" | tr '[:upper:]' '[:lower:]'
    else
        echo "unknown"
    fi
}

# Build target triple (Rust-style)
build_target_triple() {
    local os arch libc triple
    os=$(detect_os)
    arch=$(detect_arch)
    libc=$(detect_libc)
    
    # Construct Rust-style target triple
    case "$os" in
        linux)
            if [[ "$libc" == "musl" ]]; then
                triple="${arch}-unknown-linux-musl"
            else
                triple="${arch}-unknown-linux-gnu"
            fi
            ;;
        macos)
            triple="${arch}-apple-darwin"
            ;;
        freebsd)
            triple="${arch}-unknown-freebsd"
            ;;
        openbsd)
            triple="${arch}-unknown-openbsd"
            ;;
        netbsd)
            triple="${arch}-unknown-netbsd"
            ;;
        windows)
            triple="${arch}-pc-windows-msvc"
            ;;
        *)
            triple="${arch}-unknown-${os}"
            ;;
    esac
    
    echo "$triple"
}

# Display system info
display_system_info() {
    local os arch libc init priv pkg distro triple
    os=$(detect_os)
    arch=$(detect_arch)
    libc=$(detect_libc)
    init=$(detect_init_system)
    priv=$(detect_privilege)
    pkg=$(detect_package_manager)
    triple=$(build_target_triple)
    
    if [[ "$os" == "linux" ]]; then
        distro=$(get_linux_distro)
    else
        distro="N/A"
    fi
    
    echo -e "${BLUE}=== System Detection ===${NC}"
    echo -e "${GREEN}OS:${NC}              $os"
    echo -e "${GREEN}Architecture:${NC}    $arch"
    echo -e "${GREEN}LibC:${NC}            $libc"
    echo -e "${GREEN}Distribution:${NC}    $distro"
    echo -e "${GREEN}Init System:${NC}     $init"
    echo -e "${GREEN}Privilege:${NC}       $priv"
    echo -e "${GREEN}Package Mgr:${NC}     $pkg"
    echo -e "${GREEN}Target Triple:${NC}   $triple"
    echo ""
}

# Export for use by other scripts
export_system_env() {
    export GENOME_OS=$(detect_os)
    export GENOME_ARCH=$(detect_arch)
    export GENOME_LIBC=$(detect_libc)
    export GENOME_INIT=$(detect_init_system)
    export GENOME_PRIV=$(detect_privilege)
    export GENOME_PKG=$(detect_package_manager)
    export GENOME_TARGET=$(build_target_triple)
    
    if [[ "$GENOME_OS" == "linux" ]]; then
        export GENOME_DISTRO=$(get_linux_distro)
    fi
}

# Main execution (when called directly)
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    export_system_env
    display_system_info
    
    # Return 0 if system is supported, 1 otherwise
    if [[ "$GENOME_OS" == "unknown" ]] || [[ "$GENOME_ARCH" == "unknown" ]]; then
        echo -e "${RED}ERROR: Unknown or unsupported system${NC}" >&2
        exit 1
    fi
    
    exit 0
fi

