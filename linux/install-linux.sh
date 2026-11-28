#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Compilando Blue Pages em modo release..."
cd "$PROJECT_DIR"
cargo build --release

echo "==> Instalando binário em ~/.local/bin/blue-pages..."
mkdir -p "$HOME/.local/bin"
rm -f "$HOME/.local/bin/blue-pages"
cp "$PROJECT_DIR/target/release/blue-pages" "$HOME/.local/bin/blue-pages"
chmod +x "$HOME/.local/bin/blue-pages"

if [ -f "$PROJECT_DIR/libpdfium.so" ]; then
    cp "$PROJECT_DIR/libpdfium.so" "$HOME/.local/bin/libpdfium.so"
    echo "==> libpdfium.so copiado para ~/.local/bin/"
fi

echo "==> Instalando ícone da ararinha-azul..."
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
ICON_PATH="$ICON_DIR/blue-pages.png"
mkdir -p "$ICON_DIR"
cp "$PROJECT_DIR/assets/blue_pages_icon.png" "$ICON_PATH"

echo "==> Instalando atalho no menu de aplicativos..."
APP_DIR="$HOME/.local/share/applications"
mkdir -p "$APP_DIR"
sed -e "s|^Exec=.*|Exec=$HOME/.local/bin/blue-pages %f|"     -e "s|^Icon=.*|Icon=$ICON_PATH|"     "$PROJECT_DIR/linux/blue-pages.desktop" > "$APP_DIR/blue-pages.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo "==> Pronto! O Blue Pages já está disponível no menu de aplicativos com o ícone da ararinha-azul."
