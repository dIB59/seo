#!/bin/bash
# Build lighthouse-runner binaries for different platforms
# Usage: ./build.sh [platform]
# Platforms: macos-arm64, macos-x64, linux-x64, windows-x64, all (default)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_deps() {
    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed"
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        log_error "npm is not installed"
        exit 1
    fi
    
    if [ ! -d "node_modules" ]; then
        log_info "Installing dependencies..."
        npm install
    fi
}

build_macos_arm64() {
    log_info "Building for macOS ARM64..."
    npx @yao-pkg/pkg index-v11.js \
        --target node18-macos-arm64 \
        --output lighthouse-runner-aarch64-apple-darwin \
        --compress GZip \
        --config package.json
    log_info "Built: lighthouse-runner-aarch64-apple-darwin"
}

build_macos_x64() {
    log_info "Building for macOS x64..."
    npx @yao-pkg/pkg index-v11.js \
        --target node18-macos-x64 \
        --output lighthouse-runner-x86_64-apple-darwin \
        --compress GZip \
        --config package.json
    log_info "Built: lighthouse-runner-x86_64-apple-darwin"
}

build_linux_x64() {
    log_info "Building for Linux x64..."
    npx @yao-pkg/pkg index-v11.js \
        --target node18-linux-x64 \
        --output lighthouse-runner-x86_64-unknown-linux-gnu \
        --compress GZip \
        --config package.json
    log_info "Built: lighthouse-runner-x86_64-unknown-linux-gnu"
}

build_windows_x64() {
    log_info "Building for Windows x64..."
    npx @yao-pkg/pkg index-v11.js \
        --target node18-win-x64 \
        --output lighthouse-runner-x86_64-pc-windows-msvc.exe \
        --compress GZip \
        --config package.json
    log_info "Built: lighthouse-runner-x86_64-pc-windows-msvc.exe"
}

build_all() {
    build_macos_arm64
    build_macos_x64
    build_linux_x64
    build_windows_x64
}

# Main
check_deps

PLATFORM="${1:-all}"

case "$PLATFORM" in
    macos-arm64)
        build_macos_arm64
        ;;
    macos-x64)
        build_macos_x64
        ;;
    linux-x64)
        build_linux_x64
        ;;
    windows-x64)
        build_windows_x64
        ;;
    all)
        build_all
        ;;
    *)
        log_error "Unknown platform: $PLATFORM"
        echo "Usage: $0 [macos-arm64|macos-x64|linux-x64|windows-x64|all]"
        exit 1
        ;;
esac

log_info "Build complete!"
ls -lh lighthouse-runner-* 2>/dev/null || true
