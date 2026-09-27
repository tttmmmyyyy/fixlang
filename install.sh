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

# Succeed when the release tag `$1` names a pre-release, i.e. carries a suffix after `-`
# (`v1.5.0-rc.1`).
# Must stay in sync with the `prerelease:` input in .github/workflows/release.yml.
is_prerelease() {
    case "$1" in
        *-*) return 0 ;;
        *) return 1 ;;
    esac
}

# Sort release tags read from stdin, newest first, in semver order: `v1.5.0` comes before
# `v1.5.0-rc.1`, which comes before `v1.5.0-beta.10`, which comes before `v1.5.0-beta.9`.
# A pre-release suffix is a word (`alpha` < `beta` < `rc`) optionally followed by `.` and a number.
sort_versions() {
    awk '{
        tag = $0; v = tag; sub(/^v/, "", v)
        pre = ""; i = index(v, "-")
        if (i > 0) { pre = substr(v, i + 1); v = substr(v, 1, i - 1) }
        split(v, core, ".")
        if (pre == "") { is_full = 1; word = "-"; num = 0 }
        else {
            is_full = 0; j = index(pre, ".")
            if (j > 0) { word = substr(pre, 1, j - 1); num = substr(pre, j + 1) + 0 }
            else { word = pre; num = 0 }
        }
        printf "%d %d %d %d %s %d %s\n", core[1] + 0, core[2] + 0, core[3] + 0, is_full, word, num, tag
    }' | LC_ALL=C sort -k1,1nr -k2,2nr -k3,3nr -k4,4nr -k5,5r -k6,6nr | awk '{ print $7 }'
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
RELEASES_JSON="$(fetch "https://api.github.com/repos/${REPO}/releases?per_page=100")"
VERSIONS="$(printf '%s' "$RELEASES_JSON" | grep '"tag_name"' | sed 's/.*"tag_name":[ ]*"\([^"]*\)".*/\1/' | sort_versions)"

if [ -z "$VERSIONS" ]; then
    err "Failed to retrieve release information. Check your internet connection."
fi

# The default is the newest release without a pre-release suffix.
DEFAULT_VERSION="$(printf '%s\n' "$VERSIONS" | while IFS= read -r v; do
    if ! is_prerelease "$v"; then
        say "$v"
        break
    fi
done)"
# A repository with no stable release yet offers its newest pre-release.
if [ -z "$DEFAULT_VERSION" ]; then
    DEFAULT_VERSION="$(printf '%s\n' "$VERSIONS" | head -n1)"
fi
TOTAL="$(echo "$VERSIONS" | wc -l | tr -d ' ')"

say "Available versions:"
printf '%s\n' "$VERSIONS" | head -n10 | while IFS= read -r v; do
    if is_prerelease "$v"; then
        say "  ${v} (pre-release)"
    else
        say "  ${v}"
    fi
done
if [ "$TOTAL" -gt 10 ]; then
    say "  ... (${TOTAL} versions total)"
fi

say ""
if [ "$NON_INTERACTIVE" = "1" ]; then
    VERSION="$DEFAULT_VERSION"
    say "Version to install [${DEFAULT_VERSION}]: ${VERSION} (non-interactive, using default)"
else
    printf "Version to install [%s]: " "$DEFAULT_VERSION"
    read -r VERSION_INPUT </dev/tty
    VERSION="${VERSION_INPUT:-$DEFAULT_VERSION}"
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

# Download into a temporary file beside the target and move it into place, so a failed or
# interrupted download leaves any installed binary as it was, and a running one can be replaced.
# The download tool creates the file, so it gets the mode the umask gives a new file.
DOWNLOAD_PATH="${INSTALL_DIR}/.${BINARY_NAME}.download.$$"
trap 'rm -f "$DOWNLOAD_PATH"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

if ! download_to "$DOWNLOAD_URL" "$DOWNLOAD_PATH"; then
    err "Download failed. Version '${VERSION}' may have no pre-built binary for ${TARGET}, or the connection failed."
fi

chmod +x "$DOWNLOAD_PATH"
mv -f "$DOWNLOAD_PATH" "$INSTALL_PATH"

say "Installed: ${INSTALL_PATH}"

# Advise the user to add the install directory to PATH if needed.
case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
        say ""
        say "${INSTALL_DIR} is already in your PATH."
        say "Done! Run 'fix version' to verify the installation."
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
                say ""
                say "Then run 'fix version' to verify the installation."
                ;;
        esac
        ;;
esac

say ""
