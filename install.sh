#!/usr/bin/env bash
set -euo pipefail

APP_NAME="ti"
INSTALL_DIR="${HOME}/.local/bin"
BIN_PATH="target/release/${APP_NAME}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "error: cargo is required to build ${APP_NAME}" >&2
  echo "install Rust from https://rustup.rs/ and run this script again" >&2
  exit 1
fi

echo "Building ${APP_NAME}..."
cargo build --release

if [[ ! -x "${BIN_PATH}" ]]; then
  echo "error: expected binary was not produced at ${BIN_PATH}" >&2
  exit 1
fi

mkdir -p "${INSTALL_DIR}"

if [[ -e "${INSTALL_DIR}/${APP_NAME}" ]]; then
  echo "Replacing existing ${INSTALL_DIR}/${APP_NAME}..."
else
  echo "Installing to ${INSTALL_DIR}/${APP_NAME}..."
fi

cp "${BIN_PATH}" "${INSTALL_DIR}/${APP_NAME}"
chmod 755 "${INSTALL_DIR}/${APP_NAME}"

echo "Installed ${APP_NAME} to ${INSTALL_DIR}/${APP_NAME}"

case ":${PATH:-}:" in
  *":${INSTALL_DIR}:"*) ;;
  *)
    echo "warning: ${INSTALL_DIR} is not in your PATH" >&2
    echo 'add this to your shell profile: export PATH="$HOME/.local/bin:$PATH"' >&2
    ;;
esac
