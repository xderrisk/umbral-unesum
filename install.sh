#!/usr/bin/env bash
set -euo pipefail

DIR="${HOME}/.umbral"
REPO="xderrisk/umbral-unesum"
PRERELEASE="${1:-}"

if [ "$PRERELEASE" = "--prerelease" ]; then
    URL=$(curl -s "https://api.github.com/repos/$REPO/releases" |
        jq -r '[.[] | select(.prerelease) | .assets[] |
                select(.name == "umbral-arm64.tar.gz") |
                .browser_download_url] | first // empty')
else
    URL=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" |
        jq -r '.assets[] |
               select(.name == "umbral-arm64.tar.gz") |
               .browser_download_url // empty')
fi

if [ -z "$URL" ]; then
    echo "Error: no se encontró umbral-arm64.tar.gz" >&2
    exit 1
fi

curl -#L "$URL" -o /tmp/umbral-arm64.tar.gz
mkdir -p "$DIR"
tar -xzf /tmp/umbral-arm64.tar.gz -C "$DIR"
rm -f /tmp/umbral-arm64.tar.gz

DESKTOP="${DIR}/umbral.desktop"
if [ -f "$DESKTOP" ]; then
    AUTOSTART="${HOME}/.config/autostart"
    mkdir -p "$AUTOSTART"
    sed -i "s|^Exec=.*|Exec=${DIR}/umbral|" "$DESKTOP"
    sed -i "s|^Icon=.*|Icon=${DIR}/umbral.png|" "$DESKTOP"
    cp "$DESKTOP" "$AUTOSTART/"
fi

echo "Umbral instalado en ${DIR} y configurado al inicio."
