#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/lib"
CHECKSUM_FILE="$SCRIPT_DIR/checksums.sha256"

WIN_COMMIT="954696046936"
LINUX_COMMIT="ec8d9ce82dd9"

WIN_URL="https://github.com/unicorn-bi/Unicorn-Hybrid-Black-Windows-APIs/raw/${WIN_COMMIT}/c-api/Lib/Unicorn.dll"
LINUX_URL="https://github.com/unicorn-bi/Unicorn-Suite-Hybrid-Black/raw/${LINUX_COMMIT}/Unicorn%20Linux%20C%20API/x64/Lib/libunicorn.so"

SHA256=$(command -v sha256sum &>/dev/null && echo "sha256sum" || echo "shasum -a 256")

verify() {
    local file="$1" base; base="$(basename "$file")"
    local expected; expected="$(grep "  ${base}\$" "$CHECKSUM_FILE" | awk '{print $1}')"
    [ -z "$expected" ] && { echo "✗ No checksum for $base"; return 1; }
    local actual; actual="$($SHA256 "$file" | awk '{print $1}')"
    [ "$actual" != "$expected" ] && { echo "✗ CHECKSUM MISMATCH: $base"; echo "  expected=$expected"; echo "  actual=$actual"; rm -f "$file"; return 1; }
    echo "✓ Verified: $base"
}

download() {
    local url="$1" dest="$2"
    [ -f "$dest" ] && verify "$dest" 2>/dev/null && return 0
    echo "Downloading $(basename "$dest")..."
    mkdir -p "$(dirname "$dest")"
    curl -fSL --progress-bar "$url" -o "$dest"
    verify "$dest"
}

case "${1:-auto}" in
    windows|win) mkdir -p "$LIB_DIR/windows"; download "$WIN_URL" "$LIB_DIR/windows/Unicorn.dll" ;;
    linux)       mkdir -p "$LIB_DIR/linux";   download "$LINUX_URL" "$LIB_DIR/linux/libunicorn.so" ;;
    all)         $0 windows; $0 linux ;;
    auto)
        case "$(uname -s)" in
            Linux)  $0 linux ;;
            MINGW*|MSYS*|CYGWIN*) $0 windows ;;
            Darwin) echo "⚠ No macOS Unicorn library available. Windows and Linux only."; exit 1 ;;
            *) echo "Unknown OS"; exit 1 ;;
        esac ;;
    *) echo "Usage: $0 {windows|linux|all|auto}"; exit 1 ;;
esac
echo "Done. Libraries in: $LIB_DIR/"
