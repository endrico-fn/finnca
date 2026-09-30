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
  curl -sSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null \
    | grep '"tag_name"' | head -n 1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || true
}

install_finnca() {
  _bold "Installing ${APP_NAME}..."

  _require_cmd curl

  local version="${1:-}"
  local local_appimage=""
  
  # Cek jika ada biner AppImage lokal di folder project
  if [[ -f "./src-tauri/target/release/bundle/appimage/${APP_NAME}_0.1.0_amd64.AppImage" ]]; then
    local_appimage="./src-tauri/target/release/bundle/appimage/${APP_NAME}_0.1.0_amd64.AppImage"
  elif ls ./src-tauri/target/release/bundle/appimage/*.AppImage &>/dev/null; then
    local_appimage="$(ls ./src-tauri/target/release/bundle/appimage/*.AppImage | head -n 1)"
  fi

  if [[ "${version}" == "--system" || -z "${version}" ]]; then
    version="$(_get_latest_version)"
  fi

  if [[ -z "${version}" && -z "${local_appimage}" ]]; then
    _err "Belum ada rilis resmi yang dipublish di https://github.com/${REPO}/releases."
    _info "Pastikan rilis GitHub Actions sudah selesai dibuat."
    exit 1
  fi

  local ver_num="${version#v}"
  if [[ -z "${ver_num}" ]]; then
    ver_num="0.1.0"
  fi

  _info "Target Version: ${version:-v0.1.0 (local)}"
  _info "Destination   : ${INSTALL_DIR}"

  local tmp_dir=""
  tmp_dir="$(mktemp -d)"
  trap '[[ -n "${tmp_dir:-}" ]] && rm -rf "${tmp_dir}"' EXIT RETURN

  local appimage_path="${tmp_dir}/${APP_NAME}.AppImage"

  if [[ -n "${local_appimage}" && -f "${local_appimage}" && -z "${version}" ]]; then
    _info "Menggunakan biner AppImage lokal (${local_appimage})..."
    cp "${local_appimage}" "${appimage_path}"
  else
    local appimage_url="https://github.com/${REPO}/releases/download/${version}/${APP_NAME}_${ver_num}_amd64.AppImage"
    _info "Downloading AppImage from GitHub Releases..."
    if ! curl -fSL --progress-bar "${appimage_url}" -o "${appimage_path}"; then
      _err "Gagal mengunduh AppImage dari ${appimage_url}"
      _info "Kemungkinan rilis ${version} belum memiliki aset AppImage atau belum dipublish."
      exit 1
    fi
  fi

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

  _ok "Finnca ${version:-v0.1.0} successfully installed!"
  if [[ "${BIN_LINK}" == "${HOME}/.local/bin/${APP_NAME}" ]]; then
    if [[ ":${PATH}:" != *":${HOME}/.local/bin:"* ]]; then
      _info "Notice: Add ~/.local/bin to your PATH to run 'finnca' directly from terminal."
    fi
  fi
}

install_finnca "$@"
