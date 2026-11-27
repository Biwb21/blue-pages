#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "==> Compilando Blue Pages em modo release..."
cd "$PROJECT_DIR"
cargo build --release

echo "==> Instalando binário em ~/.local/bin/blue-pages..."
mkdir -p "$HOME/.local/bin"
cp "$PROJECT_DIR/target/release/blue-pages" "$HOME/.local/bin/blue-pages"
chmod +x "$HOME/.local/bin/blue-pages"

# Se houver libpdfium.so na raiz do projeto, copia junto do binário
if [ -f "$PROJECT_DIR/libpdfium.so" ]; then
    cp "$PROJECT_DIR/libpdfium.so" "$HOME/.local/bin/libpdfium.so"
    echo "==> libpdfium.so copiado para ~/.local/bin/"
fi

echo "==> Instalando ícone da ararinha-azul..."
ICON_DIR="$HOME/.local/share/icons/hicolor/256x256/apps"
mkdir -p "$ICON_DIR"
cp "$PROJECT_DIR/assets/blue_pages_icon.png" "$ICON_DIR/blue-pages.png"

echo "==> Instalando atalho no menu de aplicativos..."
APP_DIR="$HOME/.local/share/applications"
mkdir -p "$APP_DIR"
sed "s|^Exec=.*|Exec=$HOME/.local/bin/blue-pages %f|" \
    "$PROJECT_DIR/linux/blue-pages.desktop" > "$APP_DIR/blue-pages.desktop"

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APP_DIR" || true
fi

echo "==> Pronto! O Blue Pages já está disponível no menu de aplicativos."
