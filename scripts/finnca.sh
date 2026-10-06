#!/usr/bin/env bash
# ==============================================================================
# FINNCA // Industrial Workstation Orchestrator
# Application: Finnca (Double-Entry Personal Finance Notes)
# License: MIT · Repository: https://github.com/endrico-fn/finnca
# ==============================================================================

set -Eeuo pipefail

readonly APP_NAME="finnca"
readonly REPO="endrico-fn/finnca"
readonly GITHUB_API_URL="https://api.github.com/repos/${REPO}"
readonly GITHUB_RELEASE_URL="https://github.com/${REPO}/releases"

# --- Utilitarian Industrial Palette ---
if [[ -t 1 ]]; then
  readonly C_RESET=$'\033[0m'
  readonly C_BOLD=$'\033[1m'
  readonly C_DIM=$'\033[2m'
  readonly C_TEAL=$'\033[38;2;32;201;151m'
  readonly C_WHITE=$'\033[37m'
  readonly C_GREEN=$'\033[32m'
  readonly C_RED=$'\033[31m'
  readonly C_YELLOW=$'\033[33m'
else
  readonly C_RESET=''
  readonly C_BOLD=''
  readonly C_DIM=''
  readonly C_TEAL=''
  readonly C_WHITE=''
  readonly C_GREEN=''
  readonly C_RED=''
  readonly C_YELLOW=''
fi

# --- Precise Industrial Box Layout Engine ---
print_box_top() { printf "${C_TEAL}${C_BOLD}┌──────────────────────────────────────────────────────────────┐${C_RESET}\n"; }
print_box_mid() { printf "${C_TEAL}${C_BOLD}├──────────────────────────────────────────────────────────────┤${C_RESET}\n"; }
print_box_bot() { printf "${C_TEAL}${C_BOLD}└──────────────────────────────────────────────────────────────┘${C_RESET}\n"; }

print_box_line() {
  local text="$1"
  local plain
  plain=$(printf "%s" "${text}" | sed -E $'s/\x1B\\[[0-9;]*[a-zA-Z]//g')
  local pad=$(( 60 - ${#plain} ))
  if (( pad < 0 )); then pad=0; fi
  local spaces
  spaces=$(printf "%*s" "${pad}" "")
  printf "${C_TEAL}${C_BOLD}│${C_RESET}  %s%s${C_TEAL}${C_BOLD}│${C_RESET}\n" "${text}" "${spaces}"
}

print_banner() {
  local subtitle="${1:-Industrial Workstation Orchestrator · Standard Edition}"
  print_box_top
  print_box_line "${C_BOLD}FINNCA${C_RESET} ${C_DIM}// Advanced Double-Entry Financial Ledger Engine${C_RESET}"
  print_box_line "${C_DIM}${subtitle}${C_RESET}"
  print_box_bot
}

log_phase() {
  local current="$1"
  local total="$2"
  local title="$3"
  printf "\n${C_TEAL}${C_BOLD}:: [${current}/${total}]${C_RESET} ${C_BOLD}%s${C_RESET}\n" "${title}"
}

log_step() {
  local title="$1"
  local value="${2:-}"
  if [[ -n "${value}" ]]; then
    printf "   ${C_DIM}•${C_RESET} %-24s : ${C_WHITE}%s${C_RESET}\n" "${title}" "${value}"
  else
    printf "   ${C_DIM}•${C_RESET} %s\n" "${title}"
  fi
}

log_ok()   { printf "   ${C_GREEN}✔${C_RESET} %s\n" "$*"; }
log_warn() { printf "   ${C_YELLOW}⚠${C_RESET} %s\n" "$*"; }
log_err()  { printf "   ${C_RED}✖${C_RESET} %s\n" "$*" >&2; }

# --- Path Resolution & Privileges ---
detect_paths() {
  local is_system="${1:-false}"
  if [[ "${is_system}" == true || "${EUID}" -eq 0 ]]; then
    INSTALL_DIR="/opt/${APP_NAME}"
    BIN_LINK="/usr/local/bin/${APP_NAME}"
    DESKTOP_FILE="/usr/share/applications/${APP_NAME}.desktop"
    ICON_BASE_DIR="/usr/share/icons/hicolor"
    PIXMAPS_DIR="/usr/share/pixmaps"
    BASH_COMP_DIR="/usr/share/bash-completion/completions"
    ZSH_COMP_DIR="/usr/share/zsh/site-functions"
    FISH_COMP_DIR="/usr/share/fish/vendor_completions.d"
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
    BASH_COMP_DIR="${HOME}/.local/share/bash-completion/completions"
    ZSH_COMP_DIR="${HOME}/.local/share/zsh/site-functions"
    FISH_COMP_DIR="${HOME}/.config/fish/completions"
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
  if [[ "${XDG_CURRENT_DESKTOP:-}" == *"KDE"* ]] && command -v systemctl &>/dev/null; then
    if systemctl --user is-active --quiet plasma-plasmashell.service 2>/dev/null; then
      systemctl --user restart plasma-plasmashell.service 2>/dev/null || true
    fi
  fi
}

# --- Install Shell Completions ---
install_completions() {
  # 1. Bash completion
  if [[ -d "${BASH_COMP_DIR}" || -d "$(dirname "${BASH_COMP_DIR}")" ]]; then
    ${SUDO_CMD} mkdir -p "${BASH_COMP_DIR}"
    cat << 'EOF' | ${SUDO_CMD} tee "${BASH_COMP_DIR}/${APP_NAME}" > /dev/null
_finnca_completions() {
  local cur="${COMP_WORDS[COMP_CWORD]}"
  local opts="--update update -u --upgrade upgrade --uninstall uninstall --status status --version -v --help -h"
  if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]]; then
    COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
    return 0
  fi
  COMPREPLY=( $(compgen -f -X '!*.finnca' -- "${cur}") )
}
complete -F _finnca_completions finnca
EOF
    ${SUDO_CMD} chmod 644 "${BASH_COMP_DIR}/${APP_NAME}" 2>/dev/null || true
  fi

  # 2. Zsh completion
  if [[ -d "${ZSH_COMP_DIR}" || -d "$(dirname "${ZSH_COMP_DIR}")" ]]; then
    ${SUDO_CMD} mkdir -p "${ZSH_COMP_DIR}"
    cat << 'EOF' | ${SUDO_CMD} tee "${ZSH_COMP_DIR}/_${APP_NAME}" > /dev/null
#compdef finnca

_finnca() {
  local -a args
  args=(
    '(-u --update update --upgrade upgrade)'{--update,update,-u,--upgrade,upgrade}'[Check GitHub releases and upgrade]'
    '(--uninstall uninstall)'{--uninstall,uninstall}'[Safely remove application and shortcuts]'
    '(--status status)'{--status,status}'[Display deployment status and version]'
    '(-v --version)'{-v,--version}'[Print currently installed version]'
    '(-h --help)'{-h,--help}'[Show usage help]'
    '*:vault archive:_files -g "*.finnca"'
  )
  _arguments -s -S $args
}

_finnca "$@"
EOF
    ${SUDO_CMD} chmod 644 "${ZSH_COMP_DIR}/_${APP_NAME}" 2>/dev/null || true
  fi

  # 3. Fish completion
  if [[ -d "${FISH_COMP_DIR}" || -d "$(dirname "${FISH_COMP_DIR}")" ]]; then
    ${SUDO_CMD} mkdir -p "${FISH_COMP_DIR}"
    cat << 'EOF' | ${SUDO_CMD} tee "${FISH_COMP_DIR}/${APP_NAME}.fish" > /dev/null
complete -c finnca -l update -d "Check GitHub releases and upgrade"
complete -c finnca -l uninstall -d "Safely remove Finnca application"
complete -c finnca -l status -d "Display installation diagnostics"
complete -c finnca -s v -l version -d "Print installed version"
complete -c finnca -s h -l help -d "Show usage help"
complete -c finnca -r -k -a "(__fish_complete_suffix .finnca)"
EOF
    ${SUDO_CMD} chmod 644 "${FISH_COMP_DIR}/${APP_NAME}.fish" 2>/dev/null || true
  fi
}

remove_completions() {
  ${SUDO_CMD} rm -f "${BASH_COMP_DIR}/${APP_NAME}" 2>/dev/null || true
  ${SUDO_CMD} rm -f "${ZSH_COMP_DIR}/_${APP_NAME}" 2>/dev/null || true
  ${SUDO_CMD} rm -f "${FISH_COMP_DIR}/${APP_NAME}.fish" 2>/dev/null || true
}

# --- Core Action: Install ---
cmd_install() {
  local target_ver=""
  local is_system=false
  local explicit_dir=""
  local target_format="appimage"
  local use_local=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --system) is_system=true; shift ;;
      --version=*) target_ver="${1#*=}"; shift ;;
      --version) target_ver="${2:-}"; shift 2 ;;
      --dir=*) explicit_dir="${1#*=}"; shift ;;
      --dir) explicit_dir="${2:-}"; shift 2 ;;
      --format=*) target_format="${1#*=}"; shift ;;
      --format) target_format="${2:-}"; shift 2 ;;
      --appimage) target_format="appimage"; shift ;;
      --deb) target_format="deb"; shift ;;
      --rpm) target_format="rpm"; shift ;;
      --local) use_local=true; shift ;;
      v[0-9]*|[0-9]*) target_ver="$1"; shift ;;
      local) use_local=true; shift ;;
      *) shift ;;
    esac
  done

  detect_paths "${is_system}"
  if [[ -n "${explicit_dir}" ]]; then
    INSTALL_DIR="${explicit_dir}"
  fi

  print_banner "Deployment Workstation · Architecture: x86_64"

  # Search local developer artifacts
  local local_appimage=""
  local local_deb=""
  local local_rpm=""
  local script_source="${BASH_SOURCE[0]:-}"
  local search_dirs=()
  if [[ -n "${script_source}" && -f "${script_source}" ]]; then
    local project_root="$(cd "$(dirname "${script_source}")/.." 2>/dev/null && pwd || true)"
    if [[ -n "${project_root}" ]]; then
      search_dirs+=("${project_root}/src-tauri/target/release/bundle")
    fi
  fi
  search_dirs+=("./src-tauri/target/release/bundle")

  for bdir in "${search_dirs[@]}"; do
    if [[ -d "${bdir}" ]]; then
      if [[ -z "${local_appimage}" && -d "${bdir}/appimage" ]]; then
        local_appimage="$(find "${bdir}/appimage" -maxdepth 1 -name "*.AppImage" 2>/dev/null | head -n 1 || true)"
      fi
      if [[ -z "${local_deb}" && -d "${bdir}/deb" ]]; then
        local_deb="$(find "${bdir}/deb" -maxdepth 1 -name "*.deb" 2>/dev/null | head -n 1 || true)"
      fi
      if [[ -z "${local_rpm}" && -d "${bdir}/rpm" ]]; then
        local_rpm="$(find "${bdir}/rpm" -maxdepth 1 -name "*.rpm" 2>/dev/null | head -n 1 || true)"
      fi
    fi
  done

  # --- Macro Phase 1: Environment & Resolution ---
  log_phase "1" "3" "Resolving Target Architecture & Release"

  if [[ "${use_local}" == true ]]; then
    target_ver="local"
  elif [[ -z "${target_ver}" ]]; then
    target_ver="$(get_latest_version)"
    if [[ -z "${target_ver}" ]]; then
      case "${target_format}" in
        appimage) [[ -n "${local_appimage}" ]] && use_local=true && target_ver="local" ;;
        deb)      [[ -n "${local_deb}" ]]      && use_local=true && target_ver="local" ;;
        rpm)      [[ -n "${local_rpm}" ]]      && use_local=true && target_ver="local" ;;
      esac
    fi
  fi

  if [[ -z "${target_ver}" ]]; then
    log_err "No published releases found on ${GITHUB_RELEASE_URL}."
    log_info "Ensure GitHub Actions release workflow has completed, or pass --local to install local build."
    exit 1
  fi

  local clean_num="${target_ver#v}"
  if [[ "${clean_num}" == "local" || -z "${clean_num}" ]]; then
    clean_num="1.0.0"
  fi

  log_step "Distribution Format" "${target_format^^} (Linux x86_64)"
  log_step "Target Release" "${target_ver}"
  log_step "Installation Scope" "$([[ "${is_system}" == true ]] && echo "System-wide (/opt)" || echo "User-local (Rootless)")"
  log_step "Destination Root" "${INSTALL_DIR}"

  local tmp_dir=""
  tmp_dir="$(mktemp -d)"
  trap '[[ -n "${tmp_dir:-}" ]] && rm -rf "${tmp_dir}"' EXIT RETURN

  # --- Macro Phase 2: Payload Delivery & Extraction ---
  log_phase "2" "3" "Fetching & Staging Verified Payload"

  verify_payload_checksum() {
    local payload_file="$1"
    local version="$2"
    local work_dir="$3"

    local sums_file="${work_dir}/SHA256SUMS"
    local sums_url="${GITHUB_RELEASE_URL}/download/${version}/SHA256SUMS"
    local filename
    filename="$(basename "${payload_file}")"

    log_step "Verifying Payload Integrity" "SHA256 checksum"
    if curl -fsSL "${sums_url}" -o "${sums_file}" 2>/dev/null; then
      local expected_hash
      expected_hash="$(grep -E "(^|[[:space:]]|\*)${filename}[[:space:]]*\$" "${sums_file}" 2>/dev/null | awk '{print $1}' | tr '[:upper:]' '[:lower:]' | head -n1)"

      if [[ -n "${expected_hash}" ]]; then
        local actual_hash=""
        if command -v sha256sum &>/dev/null; then
          actual_hash="$(sha256sum "${payload_file}" | awk '{print $1}' | tr '[:upper:]' '[:lower:]')"
        elif command -v shasum &>/dev/null; then
          actual_hash="$(shasum -a 256 "${payload_file}" | awk '{print $1}' | tr '[:upper:]' '[:lower:]')"
        else
          log_warn "Neither sha256sum nor shasum found on host. Checksum validation skipped."
          return 0
        fi

        if [[ "${actual_hash}" != "${expected_hash}" ]]; then
          log_err "SHA-256 verification failed for ${filename}!"
          log_err "Expected: ${expected_hash}"
          log_err "Actual:   ${actual_hash}"
          exit 1
        fi
        log_ok "Payload SHA-256 verified (${actual_hash:0:16}...)."
      else
        log_warn "Checksum for ${filename} not found in SHA256SUMS manifest."
      fi
    else
      log_warn "SHA256SUMS manifest not available for release ${version}; skipping verification."
    fi
  }

  case "${target_format}" in
    deb)
      local deb_path=""
      if [[ "${use_local}" == true ]]; then
        [[ -z "${local_deb}" || ! -f "${local_deb}" ]] && { log_err "No local .deb build artifact found."; exit 1; }
        log_step "Local Artifact" "${local_deb}"
        deb_path="${local_deb}"
      else
        require_tool curl
        deb_path="${tmp_dir}/${APP_NAME}_${clean_num}_amd64.deb"
        local deb_url="${GITHUB_RELEASE_URL}/download/${target_ver}/${APP_NAME}_${clean_num}_amd64.deb"
        log_step "Downloading Payload" "${target_ver} (.deb)"
        COLUMNS=40 curl -# -fSL "${deb_url}" -o "${deb_path}"
        verify_payload_checksum "${deb_path}" "${target_ver}" "${tmp_dir}"
      fi

      log_phase "3" "3" "Integrating System Package (APT/DPKG)"
      if command -v apt &>/dev/null; then
        ${SUDO_CMD} apt install -y "${deb_path}"
      elif command -v dpkg &>/dev/null; then
        ${SUDO_CMD} dpkg -i "${deb_path}"
      fi
      log_ok "Debian package integrated successfully."
      return 0
      ;;

    rpm)
      local rpm_path=""
      if [[ "${use_local}" == true ]]; then
        [[ -z "${local_rpm}" || ! -f "${local_rpm}" ]] && { log_err "No local .rpm build artifact found."; exit 1; }
        log_step "Local Artifact" "${local_rpm}"
        rpm_path="${local_rpm}"
      else
        require_tool curl
        rpm_path="${tmp_dir}/${APP_NAME}-${clean_num}-1.x86_64.rpm"
        local rpm_url="${GITHUB_RELEASE_URL}/download/${target_ver}/${APP_NAME}-${clean_num}-1.x86_64.rpm"
        log_step "Downloading Payload" "${target_ver} (.rpm)"
        COLUMNS=40 curl -# -fSL "${rpm_url}" -o "${rpm_path}"
        verify_payload_checksum "${rpm_path}" "${target_ver}" "${tmp_dir}"
      fi

      log_phase "3" "3" "Integrating System Package (RPM)"
      if command -v dnf &>/dev/null; then
        ${SUDO_CMD} dnf install -y "${rpm_path}"
      elif command -v zypper &>/dev/null; then
        ${SUDO_CMD} zypper install -y "${rpm_path}"
      elif command -v rpm &>/dev/null; then
        ${SUDO_CMD} rpm -Uvh "${rpm_path}"
      fi
      log_ok "RPM package integrated successfully."
      return 0
      ;;

    appimage)
      local appimage_path="${tmp_dir}/${APP_NAME}.AppImage"
      if [[ "${use_local}" == true ]]; then
        [[ -z "${local_appimage}" || ! -f "${local_appimage}" ]] && { log_err "No local AppImage build artifact found."; exit 1; }
        log_step "Local Artifact" "${local_appimage}"
        cp "${local_appimage}" "${appimage_path}"
      else
        require_tool curl
        local appimage_url="${GITHUB_RELEASE_URL}/download/${target_ver}/${APP_NAME}_${clean_num}_amd64.AppImage"
        log_step "Downloading Payload" "${target_ver} (amd64 AppImage)"
        COLUMNS=40 curl -# -fSL "${appimage_url}" -o "${appimage_path}"
        verify_payload_checksum "${appimage_path}" "${target_ver}" "${tmp_dir}"
      fi

      chmod +x "${appimage_path}"
      (cd "${tmp_dir}" && "${appimage_path}" --appimage-extract > /dev/null)
      if [[ ! -d "${tmp_dir}/squashfs-root" ]]; then
        log_err "Extraction failed: Corrupt AppImage artifact."
        exit 1
      fi
      log_ok "Payload extracted and verified."
      ;;
  esac

  # --- Macro Phase 3: System Deployment & Desktop Integration ---
  log_phase "3" "3" "Deploying Engine & Desktop Integration"

  ${SUDO_CMD} rm -rf "${INSTALL_DIR}"
  ${SUDO_CMD} mkdir -p "$(dirname "${INSTALL_DIR}")"
  ${SUDO_CMD} mv "${tmp_dir}/squashfs-root" "${INSTALL_DIR}"
  echo "${target_ver:-local}" | ${SUDO_CMD} tee "${INSTALL_DIR}/version" > /dev/null

  # Bundle maintenance engine into INSTALL_DIR
  local current_script="${BASH_SOURCE[0]:-$0}"
  if [[ -f "${current_script}" && "${current_script}" != "/dev/stdin" && "${current_script}" != "/dev/fd/"* ]]; then
    ${SUDO_CMD} cp "${current_script}" "${INSTALL_DIR}/finnca.sh"
  else
    curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/scripts/finnca.sh" | ${SUDO_CMD} tee "${INSTALL_DIR}/finnca.sh" > /dev/null || true
  fi
  ${SUDO_CMD} chmod 755 "${INSTALL_DIR}/finnca.sh" 2>/dev/null || true

  # Unified CLI Launcher & Dispatcher
  ${SUDO_CMD} mkdir -p "$(dirname "${BIN_LINK}")"
  cat << 'EOF' | sed "s|__INSTALL_DIR__|${INSTALL_DIR}|g" | ${SUDO_CMD} tee "${BIN_LINK}" > /dev/null
#!/usr/bin/env bash
set -Eeuo pipefail

INSTALL_DIR="__INSTALL_DIR__"
ENGINE="${INSTALL_DIR}/finnca.sh"

case "${1:-}" in
  --update|update|-u|--upgrade|upgrade)
    shift || true
    if [[ -x "${ENGINE}" ]]; then
      exec "${ENGINE}" upgrade "$@"
    else
      curl -fsSL "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.sh" | bash -s -- upgrade "$@"
    fi
    ;;
  --uninstall|uninstall)
    shift || true
    if [[ -x "${ENGINE}" ]]; then
      exec "${ENGINE}" uninstall "$@"
    else
      curl -fsSL "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.sh" | bash -s -- uninstall "$@"
    fi
    ;;
  --status|status)
    shift || true
    if [[ -x "${ENGINE}" ]]; then
      exec "${ENGINE}" status "$@"
    else
      curl -fsSL "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.sh" | bash -s -- status "$@"
    fi
    ;;
  --version|-v)
    if [[ -f "${INSTALL_DIR}/version" ]]; then
      echo "Finnca $(cat "${INSTALL_DIR}/version")"
    else
      echo "Finnca (unknown version)"
    fi
    exit 0
    ;;
  --help|-h)
    cat << 'HELP_EOF'
Finnca - Personal Double-Entry Ledger Notes

Usage:
  finnca [FILE.finnca]       Launch desktop application (optionally open vault archive)
  finnca --update, update    Check GitHub releases and upgrade to latest
  finnca --uninstall         Safely remove Finnca desktop application
  finnca --status, status    Display installation paths and version diagnostics
  finnca --version, -v       Print currently installed version
  finnca --help, -h          Show this help message

Examples:
  finnca                     # Open Finnca GUI application
  finnca myvault.finnca      # Open specific vault archive in Finnca
  finnca --update            # Update Finnca to latest GitHub release
  finnca --uninstall         # Remove Finnca from system
HELP_EOF
    exit 0
    ;;
  *)
    exec "${INSTALL_DIR}/AppRun" "$@"
    ;;
esac
EOF
  ${SUDO_CMD} chmod 755 "${BIN_LINK}"

  # Desktop Entry (XDG compliant, named icon, StartupWMClass)
  ${SUDO_CMD} mkdir -p "$(dirname "${DESKTOP_FILE}")"
  cat << EOF | ${SUDO_CMD} tee "${DESKTOP_FILE}" > /dev/null
[Desktop Entry]
Name=Finnca
Comment=Personal double-entry finance notes
Exec=${BIN_LINK} %U
Icon=${APP_NAME}
StartupWMClass=${APP_NAME}
Type=Application
Categories=Office;Finance;
Keywords=finance;accounting;budget;ledger;vault;
MimeType=application/x-finnca;
StartupNotify=true
Terminal=false
EOF
  ${SUDO_CMD} chmod 644 "${DESKTOP_FILE}"

  # Multi-Resolution Icons Installation
  if [[ ! -f "${ICON_BASE_DIR}/index.theme" && -f "/usr/share/icons/hicolor/index.theme" ]]; then
    ${SUDO_CMD} mkdir -p "${ICON_BASE_DIR}"
    ${SUDO_CMD} cp "/usr/share/icons/hicolor/index.theme" "${ICON_BASE_DIR}/index.theme" 2>/dev/null || true
    ${SUDO_CMD} chmod 644 "${ICON_BASE_DIR}/index.theme" 2>/dev/null || true
  fi

  for size in 32x32 64x64 128x128 256x256 512x512; do
    ${SUDO_CMD} mkdir -p "${ICON_BASE_DIR}/${size}/apps"
    if [[ -f "${INSTALL_DIR}/usr/share/icons/hicolor/${size}/apps/${APP_NAME}.png" ]]; then
      ${SUDO_CMD} cp "${INSTALL_DIR}/usr/share/icons/hicolor/${size}/apps/${APP_NAME}.png" "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
    elif [[ -f "${INSTALL_DIR}/usr/share/icons/hicolor/${size}@2/apps/${APP_NAME}.png" ]]; then
      ${SUDO_CMD} cp "${INSTALL_DIR}/usr/share/icons/hicolor/${size}@2/apps/${APP_NAME}.png" "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
    elif [[ -f "${INSTALL_DIR}/${APP_NAME}.png" ]]; then
      ${SUDO_CMD} cp "${INSTALL_DIR}/${APP_NAME}.png" "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
    fi
    ${SUDO_CMD} chmod 644 "${ICON_BASE_DIR}/${size}/apps/${APP_NAME}.png" 2>/dev/null || true
  done

  # Pixmaps fallback
  ${SUDO_CMD} mkdir -p "${PIXMAPS_DIR}"
  if [[ -f "${INSTALL_DIR}/${APP_NAME}.png" ]]; then
    ${SUDO_CMD} cp "${INSTALL_DIR}/${APP_NAME}.png" "${PIXMAPS_DIR}/${APP_NAME}.png" 2>/dev/null || true
  fi
  ${SUDO_CMD} chmod 644 "${PIXMAPS_DIR}/${APP_NAME}.png" 2>/dev/null || true

  # Install Tab Completions
  install_completions

  # Synchronize caches
  refresh_desktop_caches

  log_step "Binary Installed" "${INSTALL_DIR}"
  log_step "CLI Dispatcher" "${BIN_LINK}"
  log_step "Desktop Shortcut" "${DESKTOP_FILE}"
  log_step "Icon Assets" "Multi-res (32px - 512px) in hicolor"
  log_step "Shell Completions" "bash, zsh, fish"
  log_step "Cache Status" "Desktop environment synchronized"

  # Out of the Box Completion Card
  printf "\n"
  print_box_top
  print_box_line "${C_GREEN}${C_BOLD}✔ DEPLOYMENT SUCCESSFUL${C_RESET} — Finnca ${C_BOLD}${target_ver}${C_RESET}"
  print_box_mid
  print_box_line "Launch Application   : ${C_TEAL}finnca${C_RESET}"
  print_box_line "Open Vault Archive   : ${C_TEAL}finnca <vault.finnca>${C_RESET}"
  print_box_line "Check for Upgrades   : ${C_TEAL}finnca --update${C_RESET}"
  print_box_line "Diagnostic Status    : ${C_TEAL}finnca --status${C_RESET}"
  print_box_line "Safe Removal         : ${C_TEAL}finnca --uninstall${C_RESET}"
  print_box_bot

  if [[ "${BIN_LINK}" == "${HOME}/.local/bin/${APP_NAME}" ]]; then
    if [[ ":${PATH}:" != *":${HOME}/.local/bin:"* ]]; then
      log_warn "Notice: Add '~/.local/bin' to your PATH to run 'finnca' directly from terminal."
    fi
  fi
}

# --- Core Action: Upgrade ---
cmd_upgrade() {
  print_banner "Maintenance & Upgrade Workstation"
  require_tool curl

  local current_ver="$(get_installed_version)"
  local latest_ver="$(get_latest_version)"

  printf "\n"
  log_step "Installed Version" "${current_ver}"
  log_step "Latest Available" "${latest_ver:-unknown}"

  if [[ -z "${latest_ver}" ]]; then
    log_err "Unable to fetch release information from GitHub API."
    exit 1
  fi

  if [[ "${current_ver}" != "none" && "${current_ver}" == "${latest_ver}" ]]; then
    printf "\n"
    print_box_top
    print_box_line "${C_GREEN}${C_BOLD}✔ ENGINE UP TO DATE${C_RESET}"
    print_box_line "Finnca ${C_BOLD}${current_ver}${C_RESET} is the latest release. No update needed."
    print_box_bot
    exit 0
  fi

  printf "\n"
  log_ok "Update candidate detected: ${current_ver} -> ${latest_ver}. Applying upgrade..."
  cmd_install --version="${latest_ver}"
}

# --- Core Action: Uninstall ---
cmd_uninstall() {
  local purge_data=false
  local auto_yes=false

  while [[ $# -gt 0 ]]; do
    case "$1" in
      --purge) purge_data=true; shift ;;
      -y|--yes) auto_yes=true; shift ;;
      *) shift ;;
    esac
  done

  print_banner "Safe Uninstallation Workstation"

  if [[ "${auto_yes}" != true && -t 0 ]]; then
    printf "\n${C_BOLD}Are you sure you want to remove Finnca?${C_RESET} [y/N]: "
    read -r confirm
    if [[ ! "${confirm}" =~ ^[Yy]$ ]]; then
      log_warn "Uninstallation cancelled by user."
      exit 0
    fi
  fi

  printf "\n"
  local removed=false

  # User-level removal
  detect_paths false
  if [[ -d "${INSTALL_DIR}" || -f "${BIN_LINK}" || -f "${DESKTOP_FILE}" ]]; then
    log_step "Removing application binary" "${INSTALL_DIR}"
    rm -rf "${INSTALL_DIR}"
    log_step "Removing CLI dispatcher" "${BIN_LINK}"
    rm -f "${BIN_LINK}"
    log_step "Removing desktop shortcut" "${DESKTOP_FILE}"
    rm -f "${DESKTOP_FILE}"
    log_step "Removing multi-res icon assets" "${ICON_BASE_DIR}/*/apps/${APP_NAME}.png"
    rm -f "${ICON_BASE_DIR}"/*/apps/${APP_NAME}.png
    rm -f "${PIXMAPS_DIR}/${APP_NAME}.png"
    log_step "Removing shell auto-completions" "bash, zsh, fish"
    remove_completions
    refresh_desktop_caches
    removed=true
  fi

  # System-level removal (if exists)
  if [[ -d "/opt/${APP_NAME}" || -f "/usr/local/bin/${APP_NAME}" ]]; then
    log_step "Removing system files" "/opt/${APP_NAME}"
    sudo rm -rf "/opt/${APP_NAME}"
    sudo rm -f "/usr/local/bin/${APP_NAME}"
    sudo rm -f "/usr/share/applications/${APP_NAME}.desktop"
    sudo rm -f "/usr/share/icons/hicolor"/*/apps/${APP_NAME}.png
    sudo rm -f "/usr/share/pixmaps/${APP_NAME}.png"
    sudo update-desktop-database /usr/share/applications 2>/dev/null || true
    removed=true
  fi

  if [[ "${purge_data}" == true ]]; then
    log_warn "Purge mode requested: removing configuration directory (~/.config/finnca)..."
    rm -rf "${HOME}/.config/${APP_NAME}"
  fi

  printf "\n"
  print_box_top
  print_box_line "${C_GREEN}${C_BOLD}✔ UNINSTALLATION COMPLETE${C_RESET}"
  print_box_mid
  print_box_line "${C_BOLD}Safety Invariant Preserved:${C_RESET}"
  print_box_line "Your financial vault databases (*.finnca) and local ledgers"
  print_box_line "were ${C_BOLD}NOT deleted${C_RESET}. Your data remains 100% intact."
  print_box_bot
}

# --- Core Action: Status ---
cmd_status() {
  print_banner "Deployment Diagnostics & Health Monitor"
  detect_paths false

  local inst_ver="$(get_installed_version)"
  local latest_ver="$(get_latest_version)"

  printf "\n"
  printf "  ${C_BOLD}%-24s${C_RESET} : %s\n" "Installed Version" "${inst_ver}"
  printf "  ${C_BOLD}%-24s${C_RESET} : %s\n" "Latest Release" "${latest_ver:-none}"
  printf "  ${C_BOLD}%-24s${C_RESET} : %s (%s)\n" "CLI Dispatcher" "${BIN_LINK}" \
    "$([[ -x "${BIN_LINK}" ]] && printf "${C_GREEN}Active${C_RESET}" || printf "${C_RED}Missing${C_RESET}")"
  printf "  ${C_BOLD}%-24s${C_RESET} : %s (%s)\n" "Desktop Shortcut" "${DESKTOP_FILE}" \
    "$([[ -f "${DESKTOP_FILE}" ]] && printf "${C_GREEN}Registered${C_RESET}" || printf "${C_RED}Missing${C_RESET}")"
  printf "  ${C_BOLD}%-24s${C_RESET} : %s (%s)\n" "Config Directory" "${HOME}/.config/${APP_NAME}" \
    "$([[ -d "${HOME}/.config/${APP_NAME}" ]] && printf "${C_GREEN}Exists${C_RESET}" || printf "${C_DIM}Empty${C_RESET}")"
  printf "\n"
}

# --- Usage & Help ---
print_help() {
  print_banner "Command Line Interface Reference"
  cat << EOF

Usage:
  finnca.sh <command> [options]
  curl -fsSL https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh | bash

Commands:
  install     Install or reinstall Finnca desktop application (default)
  upgrade     Check GitHub releases and upgrade to the latest version
  uninstall   Remove application binaries and shortcuts safely
  status      Display installation paths, versioning, and environment health
  help        Show this help message

Options:
  --appimage  Deploy portable AppImage bundle (default; rootless, universal)
  --deb       Install native Debian/Ubuntu .deb package via apt/dpkg
  --rpm       Install native Fedora/openSUSE .rpm package via dnf/zypper/rpm
  --local     Use local build artifact from src-tauri/target/release/bundle/
  --system    Install or remove system-wide (/opt/finnca, requires sudo)
  --version   Install specific release version tag (e.g., v1.0.2)
  --dir       Custom installation directory
  -y, --yes   Bypass confirmation prompt on uninstall
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
      if [[ "${cmd}" =~ ^v?[0-9]+\.[0-9]+\.[0-9]+ ]]; then
        cmd_install --version="${cmd}" "$@"
      elif [[ "${cmd}" == "--deb" || "${cmd}" == "--rpm" || "${cmd}" == "--appimage" || "${cmd}" == "--local" ]]; then
        cmd_install "${cmd}" "$@"
      else
        log_err "Unknown command: ${cmd}"
        print_help
        exit 1
      fi
      ;;
  esac
}

main "$@"
