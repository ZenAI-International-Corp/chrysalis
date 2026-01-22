#!/bin/bash
# Chrysalis installer script for macOS and Linux

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Repository information
REPO="ZenAI-International-Corp/chrysalis"
BINARY_NAME="chrysalis"

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux)
            OS="linux"
            ;;
        darwin)
            OS="darwin"
            ;;
        *)
            echo -e "${RED}Error: Unsupported operating system: $os${NC}"
            exit 1
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            echo -e "${RED}Error: Unsupported architecture: $arch${NC}"
            exit 1
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    echo -e "${GREEN}Detected platform: $PLATFORM${NC}"
}

# Get latest release version
get_latest_version() {
    echo -e "${YELLOW}Fetching latest version...${NC}"
    VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    
    if [ -z "$VERSION" ]; then
        echo -e "${RED}Error: Could not fetch latest version${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Latest version: $VERSION${NC}"
}

# Download and install
install() {
    local asset_name="${BINARY_NAME}-${PLATFORM}"
    local download_url="https://github.com/$REPO/releases/download/$VERSION/${asset_name}.tar.gz"
    local tmp_dir=$(mktemp -d)
    
    echo -e "${YELLOW}Downloading Chrysalis $VERSION...${NC}"
    echo -e "${YELLOW}URL: $download_url${NC}"
    
    if ! curl -fsSL "$download_url" -o "$tmp_dir/${asset_name}.tar.gz"; then
        echo -e "${RED}Error: Failed to download Chrysalis${NC}"
        echo -e "${RED}URL: $download_url${NC}"
        rm -rf "$tmp_dir"
        exit 1
    fi
    
    echo -e "${YELLOW}Extracting...${NC}"
    tar -xzf "$tmp_dir/${asset_name}.tar.gz" -C "$tmp_dir"
    
    # Determine install location
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    elif [ -w "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    else
        INSTALL_DIR="$HOME/bin"
        mkdir -p "$INSTALL_DIR"
    fi
    
    echo -e "${YELLOW}Installing to $INSTALL_DIR...${NC}"
    
    # Try to install with sudo if needed
    if ! mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME" 2>/dev/null; then
        echo -e "${YELLOW}Need sudo privileges to install to $INSTALL_DIR${NC}"
        sudo mv "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
        sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    fi
    
    # Cleanup
    rm -rf "$tmp_dir"
    
    echo -e "${GREEN}✓ Chrysalis installed successfully!${NC}"
    
    # Check if install directory is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}Warning: $INSTALL_DIR is not in your PATH${NC}"
        echo -e "${YELLOW}Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):${NC}"
        echo -e "${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    fi
    
    # Verify installation
    if command -v chrysalis &> /dev/null; then
        echo -e "${GREEN}Chrysalis version: $(chrysalis --version)${NC}"
    else
        echo -e "${YELLOW}Please restart your shell or run: export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
    fi
}

# Main
main() {
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}   Chrysalis Installer${NC}"
    echo -e "${GREEN}========================================${NC}"
    echo ""
    
    detect_platform
    get_latest_version
    install
    
    echo ""
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}Installation complete!${NC}"
    echo -e "${GREEN}Run 'chrysalis --help' to get started${NC}"
    echo -e "${GREEN}========================================${NC}"
}

main
