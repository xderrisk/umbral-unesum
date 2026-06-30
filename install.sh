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

DESKTOP="${DIR}/umbral.desktop"
if [ -f "$DESKTOP" ]; then
    sed -i "s|^Exec=.*|Exec=${DIR}/umbral --fullscreen|; s|^Icon=.*|Icon=${DIR}/umbral.png|" "$DESKTOP"
    mkdir -p "${HOME}/.local/share/applications" "${HOME}/.config/autostart"
    cp "$DESKTOP" "${HOME}/.local/share/applications/"
    cp "$DESKTOP" "${HOME}/.config/autostart/"
fi

echo "Umbral instalado en ${DIR}."
