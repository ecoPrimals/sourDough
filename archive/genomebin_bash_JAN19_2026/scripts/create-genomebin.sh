#!/usr/bin/env bash
# genomeBin Creation Script (STANDARD)
# Creates a self-extracting genomeBin from ecoBin payloads
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

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GENOMEBIN_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
PRIMAL_NAME=""
VERSION=""
ECOBINS_DIR=""
CONFIG_FILE=""
OUTPUT_FILE=""
WRAPPER_SCRIPT="${GENOMEBIN_ROOT}/wrapper/genome-wrapper.sh"

# Usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Create a genomeBin from ecoBin payloads

OPTIONS:
    --primal NAME      Primal name (required, e.g., 'beardog')
    --version VER      Version (required, e.g., '1.0.0')
    --ecobins DIR      Directory containing ecoBin files (required)
    --config FILE      Config template file (optional)
    --output FILE      Output genomeBin file (default: PRIMAL.genome)
    --wrapper FILE     Custom wrapper script (default: standard wrapper)
    -h, --help         Show this help

EXAMPLE:
    $0 --primal beardog --version 1.0.0 \\
       --ecobins ./ecobins/ \\
       --output beardog.genome

The ecobins directory should contain files like:
    beardog-x86_64-linux-musl
    beardog-aarch64-linux-musl
    beardog-x86_64-apple-darwin
    etc.
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --primal)
            PRIMAL_NAME="$2"
            shift 2
            ;;
        --version)
            VERSION="$2"
            shift 2
            ;;
        --ecobins)
            ECOBINS_DIR="$2"
            shift 2
            ;;
        --config)
            CONFIG_FILE="$2"
            shift 2
            ;;
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --wrapper)
            WRAPPER_SCRIPT="$2"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo -e "${RED}ERROR: Unknown option: $1${NC}" >&2
            usage
            exit 1
            ;;
    esac
done

# Validate required arguments
if [[ -z "$PRIMAL_NAME" ]]; then
    echo -e "${RED}ERROR: --primal is required${NC}" >&2
    usage
    exit 1
fi

if [[ -z "$VERSION" ]]; then
    echo -e "${RED}ERROR: --version is required${NC}" >&2
    usage
    exit 1
fi

if [[ -z "$ECOBINS_DIR" ]]; then
    echo -e "${RED}ERROR: --ecobins is required${NC}" >&2
    usage
    exit 1
fi

if [[ ! -d "$ECOBINS_DIR" ]]; then
    echo -e "${RED}ERROR: ecobins directory not found: $ECOBINS_DIR${NC}" >&2
    exit 1
fi

# Set default output file
if [[ -z "$OUTPUT_FILE" ]]; then
    OUTPUT_FILE="${PRIMAL_NAME}.genome"
fi

# Check wrapper script exists
if [[ ! -f "$WRAPPER_SCRIPT" ]]; then
    echo -e "${RED}ERROR: Wrapper script not found: $WRAPPER_SCRIPT${NC}" >&2
    exit 1
fi

echo -e "${BLUE}=== Creating genomeBin ===${NC}"
echo -e "${GREEN}Primal:${NC}       $PRIMAL_NAME"
echo -e "${GREEN}Version:${NC}      $VERSION"
echo -e "${GREEN}ecoBins:${NC}      $ECOBINS_DIR"
echo -e "${GREEN}Output:${NC}       $OUTPUT_FILE"
echo ""

# Find ecoBin files
ECOBIN_FILES=()
while IFS= read -r -d '' file; do
    ECOBIN_FILES+=("$file")
done < <(find "$ECOBINS_DIR" -type f -name "${PRIMAL_NAME}-*" -print0)

if [[ ${#ECOBIN_FILES[@]} -eq 0 ]]; then
    echo -e "${RED}ERROR: No ecoBin files found matching: ${PRIMAL_NAME}-*${NC}" >&2
    echo -e "${YELLOW}Expected files like: ${PRIMAL_NAME}-x86_64-linux-musl${NC}" >&2
    exit 1
fi

echo -e "${GREEN}Found ${#ECOBIN_FILES[@]} ecoBin(s):${NC}"
for file in "${ECOBIN_FILES[@]}"; do
    local_size=$(du -h "$file" | cut -f1)
    echo "  - $(basename "$file") ($local_size)"
done
echo ""

# Create temporary directory for archive creation
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Copy ecoBins to temp directory
mkdir -p "$TEMP_DIR/ecobins"
for file in "${ECOBIN_FILES[@]}"; do
    cp "$file" "$TEMP_DIR/ecobins/"
done

# Copy config if provided
if [[ -n "$CONFIG_FILE" ]] && [[ -f "$CONFIG_FILE" ]]; then
    cp "$CONFIG_FILE" "$TEMP_DIR/config.toml"
fi

# Create metadata file
cat > "$TEMP_DIR/metadata.toml" <<EOF
[genome]
primal = "${PRIMAL_NAME}"
version = "${VERSION}"
created = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
architecture_count = ${#ECOBIN_FILES[@]}

[architectures]
EOF

# Add architecture entries
for file in "${ECOBIN_FILES[@]}"; do
    basename=$(basename "$file")
    # Extract target from filename (e.g., beardog-x86_64-linux-musl -> x86_64-linux-musl)
    target="${basename#${PRIMAL_NAME}-}"
    echo "\"$target\" = \"ecobins/$basename\"" >> "$TEMP_DIR/metadata.toml"
done

# Create the self-extracting archive
echo -e "${BLUE}Creating self-extracting archive...${NC}"

# Create tar.gz of payloads
PAYLOAD_TAR="${TEMP_DIR}/payload.tar.gz"
tar -czf "$PAYLOAD_TAR" -C "$TEMP_DIR" ecobins metadata.toml $([ -f "$TEMP_DIR/config.toml" ] && echo "config.toml" || echo "")

# Create the genomeBin: wrapper script + embedded archive
{
    # Copy wrapper script
    cat "$WRAPPER_SCRIPT"
    
    # Add marker
    echo ""
    echo "# === GENOME_PAYLOAD_BOUNDARY ==="
    echo "exit 0"
    echo "# === EMBEDDED_PAYLOAD ==="
    
    # Append tar.gz payload
    cat "$PAYLOAD_TAR"
} > "$OUTPUT_FILE"

# Make executable
chmod +x "$OUTPUT_FILE"

# Calculate checksum
CHECKSUM_FILE="${OUTPUT_FILE}.sha256"
sha256sum "$OUTPUT_FILE" | cut -d' ' -f1 > "$CHECKSUM_FILE"

# Get file size
FILE_SIZE=$(du -h "$OUTPUT_FILE" | cut -f1)

echo ""
echo -e "${GREEN}✅ genomeBin created successfully!${NC}"
echo -e "${GREEN}File:${NC}         $OUTPUT_FILE"
echo -e "${GREEN}Size:${NC}         $FILE_SIZE"
echo -e "${GREEN}Checksum:${NC}    $(cat "$CHECKSUM_FILE")"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Test: $SCRIPT_DIR/test-genomebin.sh $OUTPUT_FILE"
echo "  2. Sign: $SCRIPT_DIR/sign-genomebin.sh $OUTPUT_FILE"
echo "  3. Publish to distribution server"
echo ""

