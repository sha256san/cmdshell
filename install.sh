#!/usr/bin/env bash
set -e

REPO="sha256san/cmdshell"
INSTALL_DIR="${HOME}/.local/bin"

echo "⚡ PredictTerm Installer"
echo "========================"

# Detect Operating System
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
    linux*)
        PLATFORM="linux"
        ;;
    darwin*)
        PLATFORM="macos"
        ;;
    *)
        echo "❌ Unsupported operating system: $OS"
        exit 1
        ;;
esac

# Detect Architecture
case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        if [ "$PLATFORM" = "macos" ]; then
            TARGET_ARCH="arm64"
        else
            TARGET_ARCH="x86_64" # fallback
        fi
        ;;
    *)
        echo "❌ Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

ASSET_NAME="predictterm-${PLATFORM}-${TARGET_ARCH}.tar.gz"
echo "📦 Detected platform: ${PLATFORM} (${TARGET_ARCH})"

# Create target directory
mkdir -p "$INSTALL_DIR"

# Download URL from latest release
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${ASSET_NAME}"
TEMP_DIR="$(mktemp -d)"

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "⬇️  Downloading ${ASSET_NAME}..."
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$DOWNLOAD_URL" -o "${TEMP_DIR}/${ASSET_NAME}" || {
        # Fallback to tag download if latest is pre-release
        FALLBACK_URL="https://github.com/${REPO}/releases/download/v0.1.2/${ASSET_NAME}"
        echo "ℹ️  Retrying with release tag v0.1.2..."
        curl -fsSL "$FALLBACK_URL" -o "${TEMP_DIR}/${ASSET_NAME}"
    }
elif command -v wget >/dev/null 2>&1; then
    wget -qO "${TEMP_DIR}/${ASSET_NAME}" "$DOWNLOAD_URL" || {
        FALLBACK_URL="https://github.com/${REPO}/releases/download/v0.1.2/${ASSET_NAME}"
        wget -qO "${TEMP_DIR}/${ASSET_NAME}" "$FALLBACK_URL"
    }
else
    echo "❌ Neither curl nor wget was found. Please install curl or wget."
    exit 1
fi

echo "📂 Extracting to ${INSTALL_DIR}..."
tar -xzf "${TEMP_DIR}/${ASSET_NAME}" -C "$TEMP_DIR"
cp "${TEMP_DIR}/predictterm" "${INSTALL_DIR}/predictterm"
chmod +x "${INSTALL_DIR}/predictterm"

echo ""
echo "✅ PredictTerm successfully installed to ${INSTALL_DIR}/predictterm"
echo ""

# Check PATH
case ":$PATH:" in
    *":${INSTALL_DIR}:"*)
        echo "🎉 You can now run: predictterm"
        ;;
    *)
        echo "⚠️  ${INSTALL_DIR} is not in your PATH."
        echo "Add the following line to your ~/.bashrc or ~/.zshrc:"
        echo ""
        echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        ;;
esac
