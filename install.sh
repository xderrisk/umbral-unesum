#!/usr/bin/env bash
set -euo pipefail

DIR="${HOME}/.umbral"
REPO="xderrisk/umbral-unesum"
PRERELEASE="${1:-}"

if [ "$PRERELEASE" = "--prerelease" ]; then
    URL=$(curl -sSL "https://api.github.com/repos/$REPO/releases" |
        jq -r '[.[] | select(.prerelease) | .assets[] |
                select(.name == "umbral-arm64.tar.gz") |
                .browser_download_url] | first // empty')
else
    URL=$(curl -sSL "https://api.github.com/repos/$REPO/releases/latest" |
        jq -r '.assets[] |
               select(.name == "umbral-arm64.tar.gz") |
               .browser_download_url // empty')
fi

[ -n "$URL" ] || { echo "Error: no se encontró umbral-arm64.tar.gz" >&2; exit 1; }

TAR=$(mktemp)
curl -#L "$URL" -o "$TAR"
mkdir -p "$DIR"
tar -xzf "$TAR" -C "$DIR"
rm -f "$TAR"

DESKTOP_FILE="${DIR}/umbral.desktop"
DESKTOP_DIR=$(xdg-user-dir DESKTOP 2>/dev/null || echo "${HOME}/Desktop")
AUTOSTART_DIR="${HOME}/.config/autostart"
APPLICATION_DIR="${HOME}/.local/share/applications"
if [ -f "$DESKTOP_FILE" ]; then
    sed -i "s|^Exec=.*|Exec=${DIR}/umbral --fullscreen|" "$DESKTOP_FILE"
    sed -i "s|^Icon=.*|Icon=${DIR}/umbral.png|" "$DESKTOP_FILE"
    mkdir -p "$APPLICATION_DIR" "$AUTOSTART_DIR" "$DESKTOP_DIR"
    cp "$DESKTOP_FILE" "$APPLICATION_DIR"
    cp "$DESKTOP_FILE" "$AUTOSTART_DIR/"
    cp "$DESKTOP_FILE" "$DESKTOP_DIR/"
fi

echo "Umbral instalado en ${DIR}."
