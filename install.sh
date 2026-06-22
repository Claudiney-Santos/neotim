#!/bin/bash
set -e

APP_NAME="ti" # Altere para o nome real do binário
INSTALL_DIR="$HOME/.local/bin"

echo "Compilando..."
cargo build --release

echo "Instalando..."
mkdir -p "$INSTALL_DIR"
cp "target/release/$APP_NAME" "$INSTALL_DIR/"

echo "Pronto. Instalado em $INSTALL_DIR/$APP_NAME"
