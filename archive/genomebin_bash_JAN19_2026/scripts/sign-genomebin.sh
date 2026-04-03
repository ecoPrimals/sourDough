#!/usr/bin/env bash
# genomeBin Signing Script (STANDARD)
# Signs genomeBin with GPG for authenticity verification
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

# Default values
GENOME_FILE=""
GPG_KEY=""
FORCE=false

# Usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS] GENOME_FILE

Sign a genomeBin file with GPG

ARGUMENTS:
    GENOME_FILE        Path to the genomeBin file to sign

OPTIONS:
    --key KEY_ID       GPG key ID to use for signing (default: use default key)
    --force            Overwrite existing signature
    -h, --help         Show this help

REQUIREMENTS:
    - GPG must be installed
    - GPG key must be available for signing
    - Checksum file (.sha256) must exist

OUTPUT:
    Creates GENOME_FILE.asc (GPG signature)

EXAMPLE:
    $0 beardog.genome
    $0 --key 0x1234ABCD beardog.genome
EOF
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --key)
            GPG_KEY="$2"
            shift 2
            ;;
        --force)
            FORCE=true
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        -*)
            echo -e "${RED}ERROR: Unknown option: $1${NC}" >&2
            usage
            exit 1
            ;;
        *)
            GENOME_FILE="$1"
            shift
            ;;
    esac
done

# Validate arguments
if [[ -z "$GENOME_FILE" ]]; then
    echo -e "${RED}ERROR: GENOME_FILE is required${NC}" >&2
    usage
    exit 1
fi

if [[ ! -f "$GENOME_FILE" ]]; then
    echo -e "${RED}ERROR: File not found: $GENOME_FILE${NC}" >&2
    exit 1
fi

# Check GPG availability
if ! command -v gpg &>/dev/null; then
    echo -e "${RED}ERROR: GPG not found. Please install GPG to sign genomeBins.${NC}" >&2
    echo -e "${YELLOW}On Debian/Ubuntu: sudo apt install gnupg${NC}" >&2
    echo -e "${YELLOW}On Fedora: sudo dnf install gnupg${NC}" >&2
    echo -e "${YELLOW}On macOS: brew install gnupg${NC}" >&2
    exit 1
fi

# Check checksum file
CHECKSUM_FILE="${GENOME_FILE}.sha256"
if [[ ! -f "$CHECKSUM_FILE" ]]; then
    echo -e "${YELLOW}WARNING: Checksum file not found, creating it${NC}"
    sha256sum "$GENOME_FILE" | cut -d' ' -f1 > "$CHECKSUM_FILE"
fi

# Signature file
SIG_FILE="${GENOME_FILE}.asc"

# Check if signature already exists
if [[ -f "$SIG_FILE" ]] && [[ "$FORCE" != true ]]; then
    echo -e "${RED}ERROR: Signature already exists: $SIG_FILE${NC}" >&2
    echo -e "${YELLOW}Use --force to overwrite${NC}" >&2
    exit 1
fi

echo -e "${BLUE}=== Signing genomeBin ===${NC}"
echo -e "${GREEN}File:${NC}         $GENOME_FILE"
echo -e "${GREEN}Checksum:${NC}    $(cat "$CHECKSUM_FILE")"
if [[ -n "$GPG_KEY" ]]; then
    echo -e "${GREEN}GPG Key:${NC}     $GPG_KEY"
fi
echo ""

# Sign the file
echo -e "${BLUE}Creating GPG signature...${NC}"

if [[ -n "$GPG_KEY" ]]; then
    gpg --detach-sign --armor --default-key "$GPG_KEY" --output "$SIG_FILE" "$GENOME_FILE"
else
    gpg --detach-sign --armor --output "$SIG_FILE" "$GENOME_FILE"
fi

# Verify signature
echo -e "${BLUE}Verifying signature...${NC}"
if gpg --verify "$SIG_FILE" "$GENOME_FILE" 2>&1 | grep -q "Good signature"; then
    echo -e "${GREEN}✅ Signature verified!${NC}"
else
    echo -e "${RED}❌ Signature verification failed${NC}" >&2
    exit 1
fi

# Get key info
KEY_INFO=$(gpg --verify "$SIG_FILE" "$GENOME_FILE" 2>&1 | grep "using" || echo "Unknown")

echo ""
echo -e "${GREEN}✅ genomeBin signed successfully!${NC}"
echo -e "${GREEN}Signature:${NC}   $SIG_FILE"
echo -e "${GREEN}Key:${NC}         $KEY_INFO"
echo ""
echo -e "${YELLOW}Distribution files:${NC}"
echo "  - $GENOME_FILE"
echo "  - $CHECKSUM_FILE"
echo "  - $SIG_FILE"
echo ""
echo -e "${YELLOW}Users can verify with:${NC}"
echo "  gpg --verify $SIG_FILE $GENOME_FILE"
echo ""

