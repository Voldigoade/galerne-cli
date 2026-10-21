#!/usr/bin/env sh
# Galerne installer for Linux & macOS
# Usage: curl -sSL https://raw.githubusercontent.com/Voldigoade/galerne/main/scripts/install.sh | sh

set -eu

INSTALL_DIR="${HOME}/.local/bin"
REPO="Voldigoade/galerne"

echo "=== Galerne Installer ==="

# Check architecture and OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64) TARGET_ARCH="x86_64" ;;
    aarch64|arm64) TARGET_ARCH="aarch64" ;;
    *)
        echo "Error: Unsupported architecture $ARCH" >&2
        exit 1
        ;;
esac

echo "Detected Platform: ${OS} (${TARGET_ARCH})"

# Ensure install directory exists
mkdir -p "$INSTALL_DIR"

# Download release artifact or compile if cargo is available
if command -v galerne >/dev/null 2>&1; then
    echo "✓ Galerne already present in PATH at $(which galerne)"
elif [ -f "./target/release/galerne" ]; then
    echo "Installing from local release build..."
    cp "./target/release/galerne" "${INSTALL_DIR}/galerne"
    chmod +x "${INSTALL_DIR}/galerne"
else
    echo "Fetching latest release binary from GitHub..."
    DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/galerne-${TARGET_ARCH}-unknown-linux-gnu.tar.gz"
    TMP_DIR="$(mktemp -d)"
    trap 'rm -rf "$TMP_DIR"' EXIT

    if curl -sSL --fail "$DOWNLOAD_URL" -o "${TMP_DIR}/galerne.tar.gz"; then
        tar -xzf "${TMP_DIR}/galerne.tar.gz" -C "$TMP_DIR"
        mv "${TMP_DIR}/galerne" "${INSTALL_DIR}/galerne"
        chmod +x "${INSTALL_DIR}/galerne"
    elif command -v cargo >/dev/null 2>&1; then
        echo "Precompiled binary not found on release; compiling locally with cargo..."
        cargo install --git "https://github.com/${REPO}.git" --bin galerne
    else
        echo "Error: Unable to download precompiled binary and Cargo is not installed." >&2
        exit 1
    fi
fi

# PATH check
case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        echo "Notice: ${INSTALL_DIR} is not currently in your PATH."
        echo "Add the following line to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\${HOME}/.local/bin:\$PATH\""
        ;;
esac

# Configure shell hooks
if [ -x "${INSTALL_DIR}/galerne" ]; then
    "${INSTALL_DIR}/galerne" setup
fi
