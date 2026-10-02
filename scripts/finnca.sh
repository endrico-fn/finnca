#!/usr/bin/env bash
# ==============================================================================
# Finnca Management & Distribution Engine (Unified CLI)
# Application: Finnca (Double-Entry Personal Finance Notes)
# License: MIT
# Repository: https://github.com/endrico-fn/finnca
# ==============================================================================

set -Eeuo pipefail

readonly APP_NAME="finnca"
readonly REPO="endrico-fn/finnca"
readonly GITHUB_API_URL="https://api.github.com/repos/${REPO}"
readonly GITHUB_RELEASE_URL="https://github.com/${REPO}/releases"

# --- Colors & Terminal Formatting ---
if [[ -t 1 ]]; then
  readonly C_RESET='\033[0m'
  readonly C_BOLD='\033[1m'
  readonly C_DIM='\033[2m'
  readonly C_RED='\033[31m'
  readonly C_GREEN='\033[32m'
  readonly C_BLUE='\033[34m'
  readonly C_YELLOW='\033[33m'
  readonly C_CYAN='\033[36m'
else
  readonly C_RESET=''
  readonly C_BOLD=''
  readonly C_DIM=''
  readonly C_RED=''
  readonly C_GREEN=''
  readonly C_BLUE=''
  readonly C_YELLOW=''
  readonly C_CYAN=''
fi

log_title() { printf "${C_BOLD}%s${C_RESET}\n" "$*"; }
log_info()  { printf "  ${C_BLUE}→${C_RESET} %s\n" "$*"; }
log_ok()    { printf "  ${C_GREEN}✔${C_RESET} %s\n" "$*"; }
log_warn()  { printf "  ${C_YELLOW}⚠${C_RESET} %s\n" "$*"; }
log_err()   { printf "  ${C_RED}✖${C_RESET} %s\n" "$*" >&2; }

# --- System & Path Resolution ---
detect_paths() {
  local is_system="${1:-false}"
  if [[ "${is_system}" == true || "${EUID}" -eq 0 ]]; then
    INSTALL_DIR="/opt/${APP_NAME}"
    BIN_LINK="/usr/local/bin/${APP_NAME}"
    DESKTOP_FILE="/usr/share/applications/${APP_NAME}.desktop"
    ICON_BASE_DIR="/usr/share/icons/hicolor"
    PIXMAPS_DIR="/usr/share/pixmaps"
    SUDO_CMD=""
    if [[ "${EUID}" -ne 0 ]]; then
      SUDO_CMD="sudo"
    fi
  else
    INSTALL_DIR="${HOME}/.local/share/${APP_NAME}"
    BIN_LINK="${HOME}/.local/bin/${APP_NAME}"
    DESKTOP_FILE="${HOME}/.local/share/applications/${APP_NAME}.desktop"
    ICON_BASE_DIR="${HOME}/.local/share/icons/hicolor"
    PIXMAPS_DIR="${HOME}/.local/share/pixmaps"
    SUDO_CMD=""
  fi
}

require_tool() {
  if ! command -v "$1" &>/dev/null; then
    log_err "Required tool '$1' not found. Please install it first."
    exit 1
  fi
}

get_latest_version() {
  local version
  version="$(curl -sSL "${GITHUB_API_URL}/releases/latest" 2>/dev/null \
    | grep '"tag_name"' | head -n 1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/' || true)"
  echo "${version}"
}

get_installed_version() {
  if [[ -f "${HOME}/.local/share/${APP_NAME}/version" ]]; then
    cat "${HOME}/.local/share/${APP_NAME}/version"
  elif [[ -f "/opt/${APP_NAME}/version" ]]; then
    cat "/opt/${APP_NAME}/version"
  else
    echo "none"
  fi
}

refresh_desktop_caches() {
  log_info "Refreshing desktop environment caches..."
  if command -v update-desktop-database &>/dev/null; then
    ${SUDO_CMD} update-desktop-database "$(dirname "${DESKTOP_FILE}")" 2>/dev/null || true
  fi
  if command -v gtk-update-icon-cache &>/dev/null; then
    ${SUDO_CMD} gtk-update-icon-cache -f -t "${ICON_BASE_DIR}" 2>/dev/null || true
  fi
  if command -v kbuildsycoca6 &>/dev/null; then
    kbuildsycoca6 --noincremental 2>/dev/null || true
  elif command -v kbuildsycoca5 &>/dev/null; then
    kbuildsycoca5 --noincremental 2>/dev/null || true
  fi
}

# --- Core Action: Install ---
cmd_install() {
  local target_ver=""
  local is_system=false
  local explicit_dir=""

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --system) is_system=true; shift ;;
      --version=*) target_ver="${1#*=}"; shift ;;
      --version) target_ver="${2:-}"; shift 2 ;;
      --dir=*) explicit_dir="${1#*=}"; shift ;;
      --dir) explicit_dir="${2:-}"; shift 2 ;;
      v[0-9]*|[0-9]*) target_ver="$1"; shift ;;
      *) shift ;;
    esac
  done

  detect_paths "${is_system}"
  if [[ -n "${explicit_dir}" ]]; then
    INSTALL_DIR="${explicit_dir}"
  fi

  log_title "Finnca Deployment & Installation Workstation"
  require_tool curl

  # Check local build artifact fallback first (developer workstation convenience)
  local local_appimage=""
  local script_source="${BASH_SOURCE[0]:-}"
  if [[ -n "${script_source}" && -f "${script_source}" ]]; then
    local project_root="$(cd "$(dirname "${script_source}")/.." 2>/dev/null && pwd || true)"
    if [[ -n "${project_root}" && -f "${project_root}/src-tauri/target/release/bundle/appimage/finnca_0.1.0_amd64.AppImage" ]]; then
      local_appimage="${project_root}/src-tauri/target/release/bundle/appimage/finnca_0.1.0_amd64.AppImage"
    fi
  elif [[ -f "./src-tauri/target/release/bundle/appimage/finnca_0.1.0_amd64.AppImage" ]]; then
    local_appimage="./src-tauri/target/release/bundle/appimage/finnca_0.1.0_amd64.AppImage"
  fi

  if [[ -z "${target_ver}" ]]; then
    target_ver="$(get_latest_version)"
  fi

  if [[ -z "${target_ver}" && -z "${local_appimage}" ]]; then
    log_err "No published releases found on ${GITHUB_RELEASE_URL}."
    log_info "Ensure GitHub Actions release workflow has completed or specify version explicitly."
    exit 1
  fi

  local clean_num="${target_ver#v}"
  if [[ -z "${clean_num}" ]]; then
    clean_num="0.1.0"
  fi

  log_info "Target Version: ${target_ver:-v0.1.0 (local)}"
  log_info "Install Prefix: ${INSTALL_DIR}"
  log_info "Launcher Path : ${BIN_LINK}"

  local tmp_dir=""
  tmp_dir="$(mktemp -d)"
  trap '[[ -n "${tmp_dir:-}" ]] && rm -rf "${tmp_dir}"' EXIT RETURN

  local appimage_path="${tmp_dir}/${APP_NAME}.AppImage"

  if [[ -n "${local_appimage}" && -f "${local_appimage}" && -z "${target_ver}" ]]; then
    log_info "Using local build artifact (${local_appimage})..."
    cp "${local_appimage}" "${appimage_path}"
  else
    local appimage_url="${GITHUB_RELEASE_URL}/download/${target_ver}/${APP_NAME}_${clean_num}_amd64.AppImage"
    log_info "Downloading verified AppImage payload..."
    if ! curl -fSL --progress-bar "${appimage_url}" -o "${appimage_path}"; then
      log_err "Failed to download binary payload from ${appimage_url}"
      exit 1
    fi
  fi

  chmod +x "${appimage_path}"

  log_info "Extracting payload..."
  (cd "${tmp_dir}" && "${appimage_path}" --appimage-extract > /dev/null)

  if [[ ! -d "${tmp_dir}/squashfs-root" ]]; then
    log_err "Payload extraction failed. Corrupt AppImage artifact."
    exit 1
  fi

  log_info "Deploying application files to destination..."
  ${SUDO_CMD} rm -rf "${INSTALL_DIR}"
  ${SUDO_CMD} mkdir -p "$(dirname "${INSTALL_DIR}")"
  ${SUDO_CMD} mv "${tmp_dir}/squashfs-root" "${INSTALL_DIR}"
  echo "${target_ver:-v0.1.0}" | ${SUDO_CMD} tee "${INSTALL_DIR}/version" > /dev/null

  # Create executable launcher wrapper (preserves AppRun working context and hooks)
  log_info "Generating executable launcher wrapper..."
  ${SUDO_CMD} mkdir -p "$(dirname "${BIN_LINK}")"
  cat << EOF | ${SUDO_CMD} tee "${BIN_LINK}" > /dev/null
#!/usr/bin/env bash
exec "${INSTALL_DIR}/AppRun" "\$@"
EOF
  ${SUDO_CMD} chmod 755 "${BIN_LINK}"

  # Desktop Entry
  log_info "Registering desktop application entry..."
  ${SUDO_CMD} mkdir -p "$(dirname "${DESKTOP_FILE}")"
  
  local icon_spec="${APP_NAME}"
  if [[ "${INSTALL_DIR}" == "${HOME}/.local/share/${APP_NAME}" ]]; then
    icon_spec="${ICON_BASE_DIR}/128x128/apps/${APP_NAME}.png"
  fi

  cat << EOF | ${SUDO_CMD} tee "${DESKTOP_FILE}" > /dev/null
[Desktop Entry]
Name=Finnca
Comment=Personal double-entry finance notes
Exec=${BIN_LINK}
Icon=${icon_spec}
Type=Application
Categories=Office;Finance;
Keywords=finance;accounting;budget;ledger;vault;
StartupNotify=true
Terminal=false
EOF
  ${SUDO_CMD} chmod 644 "${DESKTOP_FILE}"

  # Multi-Resolution Icons Installation
  log_info "Installing multi-resolution icon assets..."
  if [[ ! -f "${ICON_BASE_DIR}/index.theme" && -f "/usr/share/icons/hicolor/index.theme" ]]; then
    ${SUDO_CMD} mkdir -p "${ICON_BASE_DIR}"
    ${SUDO_CMD} cp "/usr/share/icons/hicolor/index.theme" "${ICON_BASE_DIR}/index.theme" 2>/dev/null || true
    ${SUDO_CMD} chmod 644 "${ICON_BASE_DIR}/index.theme" 2>/dev/null || true
  fi

  for size in 32x32 64x64 128x128 256x256 512x512; do
    ${SUDO_CMD} mkdir -p "${ICON_BASE_DIR}/${size}/apps"
    if [[ -f "${INSTALL_DIR}/usr/share/icons/hicolor/${size}/apps/${APP_NAME}.png" ]]; then
      ${SUDO_CMD} cp "${INSTALL_DIR}/usr/share/icons/hicolor/${size}/apps/${APP_NAME}.png" "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
    elif [[ -f "${INSTALL_DIR}/${APP_NAME}.png" ]]; then
      ${SUDO_CMD} cp "${INSTALL_DIR}/${APP_NAME}.png" "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
    fi
    ${SUDO_CMD} chmod 644 "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
  done

  # Pixmaps universal fallback
  ${SUDO_CMD} mkdir -p "${PIXMAPS_DIR}"
  if [[ -f "${INSTALL_DIR}/${APP_NAME}.png" ]]; then
    ${SUDO_CMD} cp "${INSTALL_DIR}/${APP_NAME}.png" "${PIXMAPS_DIR}/${APP_NAME}.png" 2>/dev/null || true
  elif [[ -f "${ICON_BASE_DIR}/128x128/apps/${APP_NAME}.png" ]]; then
    ${SUDO_CMD} cp "${ICON_BASE_DIR}/128x128/apps/${APP_NAME}.png" "${PIXMAPS_DIR}/${APP_NAME}.png" 2>/dev/null || true
  fi
  ${SUDO_CMD} chmod 644 "${PIXMAPS_DIR}/${APP_NAME}.png" 2>/dev/null || true

  refresh_desktop_caches

  log_ok "Finnca ${target_ver:-v0.1.0} installed successfully!"
  if [[ "${BIN_LINK}" == "${HOME}/.local/bin/${APP_NAME}" ]]; then
    if [[ ":${PATH}:" != *":${HOME}/.local/bin:"* ]]; then
      log_warn "Notice: Add '~/.local/bin' to your PATH to execute 'finnca' from terminal."
    fi
  fi
}

# --- Core Action: Upgrade ---
cmd_upgrade() {
  log_title "Checking for Finnca upgrades..."
  require_tool curl

  local current_ver="$(get_installed_version)"
  local latest_ver="$(get_latest_version)"

  log_info "Installed Version: ${current_ver}"
  log_info "Latest Available : ${latest_ver:-unknown}"

  if [[ -z "${latest_ver}" ]]; then
    log_err "Unable to fetch release information from GitHub."
    exit 1
  fi

  if [[ "${current_ver}" != "none" && "${current_ver}" == "${latest_ver}" ]]; then
    log_ok "Engine is already up to date (${current_ver})."
    exit 0
  fi

  log_info "Upgrade available: ${current_ver} -> ${latest_ver}. Applying..."
  cmd_install --version="${latest_ver}"
}

# --- Core Action: Uninstall ---
cmd_uninstall() {
  local purge_data=false
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --purge) purge_data=true; shift ;;
      *) shift ;;
    esac
  done

  log_title "Uninstalling Finnca..."
  local removed=false

  # User-level removal
  detect_paths false
  if [[ -d "${INSTALL_DIR}" || -f "${BIN_LINK}" || -f "${DESKTOP_FILE}" ]]; then
    log_info "Removing user-level files (${INSTALL_DIR})..."
    rm -rf "${INSTALL_DIR}"
    rm -f "${BIN_LINK}"
    rm -f "${DESKTOP_FILE}"
    rm -f "${ICON_BASE_DIR}"/*/apps/${APP_NAME}.png
    rm -f "${PIXMAPS_DIR}/${APP_NAME}.png"
    refresh_desktop_caches
    removed=true
  fi

  # System-level removal (if exists)
  if [[ -d "/opt/${APP_NAME}" || -f "/usr/local/bin/${APP_NAME}" ]]; then
    log_info "Removing system-level files (/opt/${APP_NAME})..."
    sudo rm -rf "/opt/${APP_NAME}"
    sudo rm -f "/usr/local/bin/${APP_NAME}"
    sudo rm -f "/usr/share/applications/${APP_NAME}.desktop"
    sudo rm -f "/usr/share/icons/hicolor"/*/apps/${APP_NAME}.png
    sudo rm -f "/usr/share/pixmaps/${APP_NAME}.png"
    sudo update-desktop-database /usr/share/applications 2>/dev/null || true
    removed=true
  fi

  if [[ "${removed}" == false ]]; then
    log_warn "No active Finnca installation found."
  else
    log_ok "Finnca binaries and desktop shortcuts removed cleanly."
  fi

  if [[ "${purge_data}" == true ]]; then
    log_warn "Purge mode requested: removing configuration directory (~/.config/finnca)..."
    rm -rf "${HOME}/.config/${APP_NAME}"
    log_ok "Configuration purged."
  else
    log_info "Safety invariant preserved: Vault databases and configs in ~/.config/finnca remain intact."
  fi
}

# --- Core Action: Status ---
cmd_status() {
  log_title "Finnca Deployment Status & Diagnostics"
  detect_paths false

  local inst_ver="$(get_installed_version)"
  local latest_ver="$(get_latest_version)"

  printf "  ${C_BOLD}%-22s${C_RESET} : %s\n" "Installed Version" "${inst_ver}"
  printf "  ${C_BOLD}%-22s${C_RESET} : %s\n" "Latest Release" "${latest_ver:-none}"
  printf "  ${C_BOLD}%-22s${C_RESET} : %s (%s)\n" "Binary Location" "${BIN_LINK}" \
    "$([[ -x "${BIN_LINK}" ]] && printf "${C_GREEN}Active${C_RESET}" || printf "${C_RED}Missing${C_RESET}")"
  printf "  ${C_BOLD}%-22s${C_RESET} : %s (%s)\n" "Desktop Shortcut" "${DESKTOP_FILE}" \
    "$([[ -f "${DESKTOP_FILE}" ]] && printf "${C_GREEN}Registered${C_RESET}" || printf "${C_RED}Missing${C_RESET}")"
  printf "  ${C_BOLD}%-22s${C_RESET} : %s (%s)\n" "Config Directory" "${HOME}/.config/${APP_NAME}" \
    "$([[ -d "${HOME}/.config/${APP_NAME}" ]] && printf "${C_GREEN}Exists${C_RESET}" || printf "${C_DIM}Empty${C_RESET}")"
}

# --- Usage & Command Dispatcher ---
print_help() {
  cat << EOF
Finnca Distribution & Management Engine (Unified CLI)

Usage:
  finnca.sh <command> [options]
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/finnca.sh | bash -s -- <command>

Commands:
  install     Install or reinstall Finnca desktop application (default)
  upgrade     Check GitHub releases and upgrade to the latest version
  uninstall   Remove application binaries and shortcuts safely
  status      Display installation paths, versioning, and environment health
  help        Show this help message

Options:
  --system    Install or remove system-wide (/opt/finnca, requires sudo)
  --version   Install specific release version tag (e.g., v0.1.2)
  --dir       Custom installation directory
  --purge     (Used with uninstall) Also remove ~/.config/finnca configuration

Examples:
  bash finnca.sh install
  bash finnca.sh upgrade
  bash finnca.sh status
  bash finnca.sh uninstall
EOF
}

main() {
  local cmd="${1:-install}"
  shift || true

  case "${cmd}" in
    install)   cmd_install "$@" ;;
    upgrade)   cmd_upgrade "$@" ;;
    uninstall) cmd_uninstall "$@" ;;
    status)    cmd_status "$@" ;;
    help|-h|--help) print_help ;;
    *)
      # Fallback: if argument is a version tag like 'v0.1.2', treat as install
      if [[ "${cmd}" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        cmd_install --version="${cmd}" "$@"
      else
        log_err "Unknown command: ${cmd}"
        print_help
        exit 1
      fi
      ;;
  esac
}

main "$@"
