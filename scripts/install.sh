#!/usr/bin/env bash
set -euo pipefail

APP_NAME="finnca"
REPO="endrico-fn/finnca"

# Mode: user-level (~/.local) by default, or system-level (/opt) if root / --system
if [[ "${EUID:-$(id -u)}" -eq 0 || "${1:-}" == "--system" ]]; then
  INSTALL_DIR="/opt/${APP_NAME}"
  BIN_LINK="/usr/local/bin/${APP_NAME}"
  DESKTOP_FILE="/usr/share/applications/${APP_NAME}.desktop"
  ICON_DIR="/usr/share/icons/hicolor/128x128/apps"
  SUDO_CMD=""
  if [[ "${EUID:-$(id -u)}" -ne 0 ]]; then
    SUDO_CMD="sudo"
  fi
else
  INSTALL_DIR="${HOME}/.local/share/${APP_NAME}"
  BIN_LINK="${HOME}/.local/bin/${APP_NAME}"
  DESKTOP_FILE="${HOME}/.local/share/applications/${APP_NAME}.desktop"
  ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"
  SUDO_CMD=""
fi

_bold() { printf '\033[1m%s\033[0m\n' "$*"; }
_info() { printf '  \033[34m→\033[0m %s\n' "$*"; }
_ok()   { printf '  \033[32m✔\033[0m %s\n' "$*"; }
_err()  { printf '  \033[31m✖\033[0m %s\n' "$*" >&2; }

_require_cmd() {
  if ! command -v "$1" &>/dev/null; then
    _err "Command not found: $1 — install it first"
    exit 1
  fi
}

_get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/'
}

install_finnca() {
  _bold "Installing ${APP_NAME}..."

  _require_cmd curl

  local version="${1:-}"
  if [[ "${version}" == "--system" || -z "${version}" ]]; then
    version="$(_get_latest_version)"
  fi
  local ver_num="${version#v}"

  _info "Target Version: ${version}"
  _info "Destination   : ${INSTALL_DIR}"

  local tmp_dir
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "${tmp_dir}"' EXIT

  local appimage_url="https://github.com/${REPO}/releases/download/${version}/${APP_NAME}_${ver_num}_amd64.AppImage"
  local appimage_path="${tmp_dir}/${APP_NAME}.AppImage"

  _info "Downloading AppImage..."
  curl -fSL --progress-bar "${appimage_url}" -o "${appimage_path}"

  chmod +x "${appimage_path}"

  _info "Extracting..."
  pushd "${tmp_dir}" > /dev/null
  "${appimage_path}" --appimage-extract > /dev/null
  popd > /dev/null

  _info "Installing application files..."
  ${SUDO_CMD} rm -rf "${INSTALL_DIR}"
  ${SUDO_CMD} mkdir -p "$(dirname "${INSTALL_DIR}")"
  ${SUDO_CMD} mv "${tmp_dir}/squashfs-root" "${INSTALL_DIR}"
  echo "${version}" | ${SUDO_CMD} tee "${INSTALL_DIR}/version" > /dev/null

  _info "Creating executable symlink (${BIN_LINK})..."
  ${SUDO_CMD} mkdir -p "$(dirname "${BIN_LINK}")"
  ${SUDO_CMD} ln -sf "${INSTALL_DIR}/AppRun" "${BIN_LINK}"

  _info "Installing .desktop entry..."
  ${SUDO_CMD} mkdir -p "$(dirname "${DESKTOP_FILE}")"
  cat << EOF | ${SUDO_CMD} tee "${DESKTOP_FILE}" > /dev/null
[Desktop Entry]
Name=Finnca
Comment=Personal double-entry finance notes
Exec=${BIN_LINK}
Icon=${APP_NAME}
Type=Application
Categories=Office;Finance;
Keywords=finance;accounting;budget;
StartupNotify=true
EOF

  _info "Installing app icon..."
  ${SUDO_CMD} mkdir -p "${ICON_DIR}"
  ${SUDO_CMD} cp "${INSTALL_DIR}/usr/share/icons/hicolor/128x128/apps/${APP_NAME}.png" \
    "${ICON_DIR}/${APP_NAME}.png" 2>/dev/null || \
  ${SUDO_CMD} cp "${INSTALL_DIR}/${APP_NAME}.png" "${ICON_DIR}/${APP_NAME}.png" 2>/dev/null || true

  if command -v update-desktop-database &>/dev/null; then
    update-desktop-database "$(dirname "${DESKTOP_FILE}")" 2>/dev/null || true
  fi
  if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f "${ICON_DIR%/*/*}" 2>/dev/null || true
  fi

  _ok "Finnca ${version} successfully installed!"
  if [[ "${BIN_LINK}" == "${HOME}/.local/bin/${APP_NAME}" ]]; then
    if [[ ":${PATH}:" != *":${HOME}/.local/bin:"* ]]; then
      _info "Notice: Add ~/.local/bin to your PATH to run 'finnca' directly from terminal."
    fi
  fi
}

install_finnca "$@"
