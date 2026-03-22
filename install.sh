#!/usr/bin/env sh
# Install script for the Fix programming language compiler.
# Usage: curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/tttmmmyyyy/fixlang/main/install.sh | sh

set -e

REPO="tttmmmyyyy/fixlang"
INSTALL_DIR="${HOME}/.local/bin"
BINARY_NAME="fix"

# If stdin is not a terminal (e.g. piped via curl | sh), check whether
# /dev/tty is available for interactive prompts. If not (e.g. Docker without
# a TTY), fall back to non-interactive mode and use default values.
# Note: we intentionally do NOT do `exec </dev/tty` here because that would
# redirect the shell's own script-reading fd and break the piped execution.
# Instead, each `read` call below explicitly uses `</dev/tty`.
NON_INTERACTIVE=0
if [ ! -t 0 ]; then
    if ! (exec </dev/tty) 2>/dev/null; then
        NON_INTERACTIVE=1
    fi
fi

say() {
    printf '%s\n' "$1"
}

err() {
    say "Error: $1" >&2
    exit 1
}

# Detect the target triple for this platform.
detect_target() {
    _os="$(uname -s)"
    _arch="$(uname -m)"
    case "$_os" in
        Linux*)
            case "$_arch" in
                x86_64) echo "x86_64-unknown-linux-gnu" ;;
                *) err "No pre-built binary available for Linux/${_arch}. See Document.md for instructions on building from source: https://github.com/${REPO}/blob/main/Document.md" ;;
            esac
            ;;
        Darwin*)
            case "$_arch" in
                arm64) echo "aarch64-apple-darwin" ;;
                *) err "No pre-built binary available for macOS/${_arch}. See Document.md for instructions on building from source: https://github.com/${REPO}/blob/main/Document.md" ;;
            esac
            ;;
        *) err "Unsupported OS: ${_os}. See Document.md for instructions on building from source: https://github.com/${REPO}/blob/main/Document.md" ;;
    esac
}

# Output URL content to stdout.
fetch() {
    _url="$1"
    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSfL "$_url"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only -qO- "$_url"
    else
        err "curl or wget is required."
    fi
}

# Download URL to a file.
download_to() {
    _url="$1"
    _dest="$2"
    if command -v curl >/dev/null 2>&1; then
        curl --proto '=https' --tlsv1.2 -sSfL "$_url" -o "$_dest"
    elif command -v wget >/dev/null 2>&1; then
        wget --https-only -qO "$_dest" "$_url"
    else
        err "curl or wget is required."
    fi
}

# ---- Main ----------------------------------------------------------------

TARGET="$(detect_target)"

say ""
say "Fix Language Installer"
say "======================"
say "Platform: ${TARGET}"
say ""

# Fetch available releases from GitHub API.
say "Fetching release list from GitHub..."
RELEASES_JSON="$(fetch "https://api.github.com/repos/${REPO}/releases")"
VERSIONS="$(printf '%s' "$RELEASES_JSON" | grep '"tag_name"' | sed 's/.*"tag_name":[ ]*"\([^"]*\)".*/\1/')"

if [ -z "$VERSIONS" ]; then
    err "Failed to retrieve release information. Check your internet connection."
fi

LATEST="$(printf '%s\n' "$VERSIONS" | head -n1)"
TOTAL="$(echo "$VERSIONS" | wc -l | tr -d ' ')"

say "Available versions:"
printf '%s\n' "$VERSIONS" | head -n10 | while IFS= read -r v; do
    say "  ${v}"
done
if [ "$TOTAL" -gt 10 ]; then
    say "  ... (${TOTAL} versions total)"
fi

say ""
if [ "$NON_INTERACTIVE" = "1" ]; then
    VERSION="$LATEST"
    say "Version to install [${LATEST}]: ${VERSION} (non-interactive, using default)"
else
    printf "Version to install [%s]: " "$LATEST"
    read -r VERSION_INPUT </dev/tty
    VERSION="${VERSION_INPUT:-$LATEST}"
fi

# Basic sanity check: version tag should start with 'v'.
case "$VERSION" in
    v*) ;;
    *) err "Unexpected version format: '${VERSION}'. Expected a tag like 'v1.2.3'." ;;
esac

say ""

# Check whether fix is already installed.
INSTALL_PATH="${INSTALL_DIR}/${BINARY_NAME}"
EXISTING_IN_PATH="$(command -v "${BINARY_NAME}" 2>/dev/null || true)"

# Warn if another fix binary is found in PATH at a different location.
if [ -n "$EXISTING_IN_PATH" ] && [ "$EXISTING_IN_PATH" != "$INSTALL_PATH" ]; then
    say "Note: fix is already found in PATH at: ${EXISTING_IN_PATH}"
    say "      The new binary will be installed to: ${INSTALL_PATH}"
    say "      That existing binary will NOT be modified."
    say ""
fi

# If the install target already exists, ask before overwriting.
if [ -f "$INSTALL_PATH" ]; then
    say "fix is already installed at: ${INSTALL_PATH}"
    if [ "$NON_INTERACTIVE" = "1" ]; then
        say "Overwrite? [y/N]: N (non-interactive, skipping installation)"
        say "Installation cancelled."; exit 0
    fi
    printf "Overwrite? [y/N]: "
    read -r OVERWRITE_INPUT </dev/tty
    case "$OVERWRITE_INPUT" in
        [yY][eE][sS]|[yY]) say "" ;;
        *) say "Installation cancelled."; exit 0 ;;
    esac
fi

# Download binary from GitHub Releases.
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/fix-${VERSION}-${TARGET}"

say "Downloading fix ${VERSION}..."
say "  ${DOWNLOAD_URL}"
say ""

mkdir -p "$INSTALL_DIR"

if ! download_to "$DOWNLOAD_URL" "$INSTALL_PATH"; then
    err "Download failed. Version '${VERSION}' may not have a pre-built binary for ${TARGET}."
fi

chmod +x "$INSTALL_PATH"

say "Installed: ${INSTALL_PATH}"

# Advise the user to add the install directory to PATH if needed.
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        say ""
        say "Done! Run 'fix --version' to verify the installation."
        ;;
    *)
        case "$(basename "${SHELL:-sh}")" in
            fish)
                say ""
                say "Add the following line to ~/.config/fish/config.fish to make fix available in new shells:"
                say ""
                say "  fish_add_path \"\$HOME/.local/bin\""
                say ""
                say "Or run it now to use fix in the current session:"
                say ""
                say "  set -x PATH \"\$HOME/.local/bin\" \$PATH"
                ;;
            *)
                case "$(basename "${SHELL:-sh}")" in
                    zsh)  PROFILE="~/.zshrc" ;;
                    bash) PROFILE="~/.bashrc" ;;
                    *)    PROFILE="your shell's profile file" ;;
                esac
                say ""
                say "Add the following line to ${PROFILE} to make fix available in new shells:"
                say ""
                say "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
                say ""
                say "Or run it now to use fix in the current session:"
                say ""
                say "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
                ;;
        esac
        ;;
esac

say ""
