#!/usr/bin/env bash
set -euo pipefail

APP_NAME="finnca"

_bold() { printf '\033[1m%s\033[0m\n' "$*"; }
_info() { printf '  \033[34m→\033[0m %s\n' "$*"; }
_ok()   { printf '  \033[32m✔\033[0m %s\n' "$*"; }
_warn() { printf '  \033[33m!\033[0m %s\n' "$*"; }

_bold "Uninstalling ${APP_NAME}..."

removed_any=false

# 1. Check user-level install (~/.local)
USER_DIR="${HOME}/.local/share/${APP_NAME}"
USER_BIN="${HOME}/.local/bin/${APP_NAME}"
USER_DESKTOP="${HOME}/.local/share/applications/${APP_NAME}.desktop"
USER_ICON="${HOME}/.local/share/icons/hicolor/128x128/apps/${APP_NAME}.png"

if [[ -d "${USER_DIR}" || -L "${USER_BIN}" || -f "${USER_DESKTOP}" ]]; then
  _info "Removing user-level files (~/.local)..."
  rm -f "${USER_BIN}"
  rm -rf "${USER_DIR}"
  rm -f "${USER_DESKTOP}"
  rm -f "${USER_ICON}"
  removed_any=true
fi

# 2. Check system-level install (/opt)
SYS_DIR="/opt/${APP_NAME}"
SYS_BIN="/usr/local/bin/${APP_NAME}"
SYS_DESKTOP="/usr/share/applications/${APP_NAME}.desktop"
SYS_ICON="/usr/share/icons/hicolor/128x128/apps/${APP_NAME}.png"

if [[ -d "${SYS_DIR}" || -L "${SYS_BIN}" || -f "${SYS_DESKTOP}" ]]; then
  _info "Removing system-level files (requires sudo)..."
  sudo rm -f "${SYS_BIN}"
  sudo rm -rf "${SYS_DIR}"
  sudo rm -f "${SYS_DESKTOP}"
  sudo rm -f "${SYS_ICON}"
  removed_any=true
fi

if [[ "${removed_any}" == false ]]; then
  _warn "Finnca installation files not found."
  exit 0
fi

if command -v update-desktop-database &>/dev/null; then
  update-desktop-database "${HOME}/.local/share/applications" 2>/dev/null || true
  sudo update-desktop-database /usr/share/applications 2>/dev/null || true
fi
if command -v kbuildsycoca6 &>/dev/null; then
  kbuildsycoca6 2>/dev/null || true
elif command -v kbuildsycoca5 &>/dev/null; then
  kbuildsycoca5 2>/dev/null || true
fi

_ok "Finnca application has been cleanly uninstalled."
_info "Note: Your encrypted vaults and configs in ~/.config/finnca remain intact."
