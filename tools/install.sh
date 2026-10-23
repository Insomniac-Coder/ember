#!/bin/sh
# `[TOOL-1]` — Ember's one-line installer for Linux x64 and macOS arm64:
#
#     curl -fsSL https://github.com/Insomniac-Coder/ember/releases/latest/download/install.sh | sh
#
# It downloads this host's toolchain archive from the release, checks it against the release's
# checksum, unpacks it into `$EMBER_HOME` (default `~/.ember`, replacing an earlier install) and puts
# `$EMBER_HOME/bin` on `PATH` in the shell's start-up files. `EMBER_VERSION=v0.1.0` installs that
# release instead of the latest.
set -eu

REPOSITORY="Insomniac-Coder/ember"
EMBER_HOME="${EMBER_HOME:-$HOME/.ember}"

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64 | Linux-amd64) host="linux-x64" ;;
    Darwin-arm64 | Darwin-aarch64) host="macos-arm64" ;;
    *) echo "ember: no archive for this host ($(uname -s) $(uname -m)); hosts: Linux x64, macOS arm64, Windows x64" >&2; exit 1 ;;
esac

if [ -n "${EMBER_VERSION:-}" ]; then
    base="https://github.com/$REPOSITORY/releases/download/$EMBER_VERSION"
else
    base="https://github.com/$REPOSITORY/releases/latest/download"
fi
archive="ember-$host.tar.gz"

fetch() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$1" -O "$2"
    else
        echo "ember: curl or wget is needed to download Ember" >&2
        exit 1
    fi
}

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
echo "downloading $archive"
fetch "$base/$archive" "$work/$archive"
fetch "$base/$archive.sha256" "$work/$archive.sha256"

expected="$(cut -d ' ' -f 1 < "$work/$archive.sha256")"
if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$work/$archive" | cut -d ' ' -f 1)"
else
    actual="$(shasum -a 256 "$work/$archive" | cut -d ' ' -f 1)"
fi
if [ "$expected" != "$actual" ]; then
    echo "ember: $archive does not match the release's checksum; nothing was installed" >&2
    exit 1
fi

tar -xzf "$work/$archive" -C "$work"
rm -rf "$EMBER_HOME"
mkdir -p "$(dirname "$EMBER_HOME")"
mv "$work/ember" "$EMBER_HOME"

line="export PATH=\"$EMBER_HOME/bin:\$PATH\""
for profile in "$HOME/.profile" "$HOME/.bashrc" "$HOME/.zshrc"; do
    case "$profile" in
        */.profile) ;;
        *) [ -f "$profile" ] || continue ;;
    esac
    if ! grep -qsF "$line" "$profile"; then
        printf '\n# Ember\n%s\n' "$line" >> "$profile"
    fi
done

echo "installed $("$EMBER_HOME/bin/ember" --version | head -n 1) in $EMBER_HOME"
echo "open a new shell (or run: $line), then: ember new hello && cd hello && ember run"
