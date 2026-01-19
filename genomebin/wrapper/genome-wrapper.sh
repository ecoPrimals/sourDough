#!/usr/bin/env bash
# genomeBin Self-Extracting Wrapper (STANDARD)
# Universal deployment wrapper for ecoPrimals genomeBins
# Part of the ecoPrimals genomeBin standard

set -euo pipefail

# Color output
if [[ -t 1 ]]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m'
else
    RED='' GREEN='' YELLOW='' BLUE='' NC=''
fi

# === CONFIGURATION ===
GENOME_VERSION="1.0.0"
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr/local}"
CONFIG_PREFIX="${CONFIG_PREFIX:-/etc}"
DATA_PREFIX="${DATA_PREFIX:-/var/lib}"

# === SYSTEM DETECTION ===
detect_system() {
    # Detect OS
    case "$(uname -s)" in
        Linux*)     GENOME_OS="linux";;
        Darwin*)    GENOME_OS="macos";;
        FreeBSD*)   GENOME_OS="freebsd";;
        *)          GENOME_OS="unknown";;
    esac
    
    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)   GENOME_ARCH="x86_64";;
        aarch64|arm64)  GENOME_ARCH="aarch64";;
        *)              GENOME_ARCH="unknown";;
    esac
    
    # Detect libc
    if ldd --version 2>&1 | grep -iq musl; then
        GENOME_LIBC="musl"
    elif ldd --version 2>&1 | grep -iq glibc; then
        GENOME_LIBC="gnu"
    elif [[ "$GENOME_OS" == "macos" ]]; then
        GENOME_LIBC="darwin"
    else
        GENOME_LIBC="unknown"
    fi
    
    # Build target triple
    if [[ "$GENOME_OS" == "linux" ]]; then
        GENOME_TARGET="${GENOME_ARCH}-unknown-linux-${GENOME_LIBC}"
    elif [[ "$GENOME_OS" == "macos" ]]; then
        GENOME_TARGET="${GENOME_ARCH}-apple-darwin"
    else
        GENOME_TARGET="${GENOME_ARCH}-unknown-${GENOME_OS}"
    fi
    
    # Check privilege
    if [[ $EUID -eq 0 ]]; then
        GENOME_PRIV="root"
    else
        GENOME_PRIV="user"
    fi
}

# === PAYLOAD EXTRACTION ===
extract_payload() {
    local temp_dir="$1"
    
    # Extract embedded archive from this script
    # The archive is appended after the "GENOME_PAYLOAD_BOUNDARY" marker
    awk '/EMBEDDED_PAYLOAD/ {found=1; next} found' "$0" | tar -xzf - -C "$temp_dir"
}

# === METADATA PARSING ===
parse_metadata() {
    local metadata_file="$1"
    
    if [[ ! -f "$metadata_file" ]]; then
        echo -e "${RED}ERROR: metadata.toml not found${NC}" >&2
        return 1
    fi
    
    # Simple TOML parsing (just extract key values we need)
    PRIMAL_NAME=$(grep '^primal =' "$metadata_file" | cut -d'"' -f2)
    PRIMAL_VERSION=$(grep '^version =' "$metadata_file" | cut -d'"' -f2)
    
    if [[ -z "$PRIMAL_NAME" ]]; then
        echo -e "${RED}ERROR: Could not parse primal name from metadata${NC}" >&2
        return 1
    fi
}

# === BINARY SELECTION ===
select_binary() {
    local ecobins_dir="$1"
    local binary_file=""
    
    # Look for matching ecoBin
    local pattern="${PRIMAL_NAME}-${GENOME_TARGET}"
    binary_file=$(find "$ecobins_dir" -name "$pattern" -type f | head -n1)
    
    if [[ -z "$binary_file" ]] || [[ ! -f "$binary_file" ]]; then
        # Try without libc suffix for macOS
        if [[ "$GENOME_OS" == "macos" ]]; then
            pattern="${PRIMAL_NAME}-${GENOME_ARCH}-apple-darwin"
            binary_file=$(find "$ecobins_dir" -name "$pattern" -type f | head -n1)
        fi
    fi
    
    if [[ -z "$binary_file" ]] || [[ ! -f "$binary_file" ]]; then
        echo -e "${RED}ERROR: No ecoBin found for ${GENOME_TARGET}${NC}" >&2
        echo -e "${YELLOW}Available ecoBins:${NC}" >&2
        ls -1 "$ecobins_dir" >&2
        return 1
    fi
    
    echo "$binary_file"
}

# === INSTALLATION ===
install_genome() {
    local extract_dir="$1"
    
    echo -e "${BLUE}=== genomeBin Installation ===${NC}"
    echo -e "${GREEN}Primal:${NC}      $PRIMAL_NAME"
    echo -e "${GREEN}Version:${NC}     $PRIMAL_VERSION"
    echo -e "${GREEN}System:${NC}      ${GENOME_OS}/${GENOME_ARCH}/${GENOME_LIBC}"
    echo -e "${GREEN}Target:${NC}      $GENOME_TARGET"
    echo -e "${GREEN}Privilege:${NC}   $GENOME_PRIV"
    echo ""
    
    # Select binary
    local binary_path
    binary_path=$(select_binary "$extract_dir/ecobins")
    
    echo -e "${GREEN}Selected:${NC}    $(basename "$binary_path")"
    echo ""
    
    # Determine install location
    local bin_dir="${INSTALL_PREFIX}/bin"
    local config_dir="${CONFIG_PREFIX}/${PRIMAL_NAME}"
    local data_dir="${DATA_PREFIX}/${PRIMAL_NAME}"
    
    # Check permissions
    if [[ ! -w "$INSTALL_PREFIX" ]] && [[ "$GENOME_PRIV" != "root" ]]; then
        echo -e "${YELLOW}WARNING: No write permission to ${INSTALL_PREFIX}${NC}" >&2
        echo -e "${YELLOW}Installing to user directories instead${NC}" >&2
        bin_dir="$HOME/.local/bin"
        config_dir="$HOME/.config/${PRIMAL_NAME}"
        data_dir="$HOME/.local/share/${PRIMAL_NAME}"
    fi
    
    # Create directories
    echo -e "${BLUE}Creating directories...${NC}"
    mkdir -p "$bin_dir" "$config_dir" "$data_dir"
    
    # Install binary
    echo -e "${BLUE}Installing binary...${NC}"
    cp "$binary_path" "${bin_dir}/${PRIMAL_NAME}"
    chmod +x "${bin_dir}/${PRIMAL_NAME}"
    
    # Install config if provided
    if [[ -f "$extract_dir/config.toml" ]]; then
        echo -e "${BLUE}Installing configuration...${NC}"
        cp "$extract_dir/config.toml" "$config_dir/config.toml"
    fi
    
    echo ""
    echo -e "${GREEN}✅ Installation complete!${NC}"
    echo ""
    echo -e "${GREEN}Installed files:${NC}"
    echo "  Binary:  ${bin_dir}/${PRIMAL_NAME}"
    echo "  Config:  ${config_dir}/"
    echo "  Data:    ${data_dir}/"
    echo ""
    
    # Verify installation
    if command -v "$PRIMAL_NAME" &>/dev/null || [[ -x "${bin_dir}/${PRIMAL_NAME}" ]]; then
        echo -e "${GREEN}Verification:${NC} Binary is executable"
        
        # Try to run doctor command if available
        if "${bin_dir}/${PRIMAL_NAME}" doctor &>/dev/null || "${bin_dir}/${PRIMAL_NAME}" --help &>/dev/null; then
            echo -e "${GREEN}Health check: PASSED${NC}"
        fi
    fi
    
    echo ""
    echo -e "${YELLOW}Next steps:${NC}"
    echo "  1. Add ${bin_dir} to your PATH if not already"
    echo "  2. Run: ${PRIMAL_NAME} --help"
    echo "  3. Configure: ${config_dir}/config.toml"
    echo ""
}

# === MAIN ===
main() {
    # Detect system
    detect_system
    
    if [[ "$GENOME_OS" == "unknown" ]] || [[ "$GENOME_ARCH" == "unknown" ]]; then
        echo -e "${RED}ERROR: Unsupported system: ${GENOME_OS}/${GENOME_ARCH}${NC}" >&2
        exit 1
    fi
    
    # Create temporary directory
    TEMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TEMP_DIR"' EXIT
    
    # Extract payload
    echo -e "${BLUE}Extracting genomeBin payload...${NC}"
    extract_payload "$TEMP_DIR"
    
    # Parse metadata
    parse_metadata "$TEMP_DIR/metadata.toml"
    
    # Install
    install_genome "$TEMP_DIR"
}

# Run main if executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi

