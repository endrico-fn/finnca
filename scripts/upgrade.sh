#!/usr/bin/env bash
set -euo pipefail

APP_NAME="finnca"
REPO="endrico-fn/finnca"

_bold() { printf '\033[1m%s\033[0m\n' "$*"; }
_info() { printf '  \033[34m→\033[0m %s\n' "$*"; }
_ok()   { printf '  \033[32m✔\033[0m %s\n' "$*"; }
_err()  { printf '  \033[31m✖\033[0m %s\n' "$*" >&2; }

_bold "Checking for ${APP_NAME} upgrades..."

_get_latest_version() {
  curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/'
}

_get_installed_version() {
  if [[ -f "${HOME}/.local/share/${APP_NAME}/version" ]]; then
    cat "${HOME}/.local/share/${APP_NAME}/version"
  elif [[ -f "/opt/${APP_NAME}/version" ]]; then
    cat "/opt/${APP_NAME}/version"
  else
    echo "none"
  fi
}

installed_version="$(_get_installed_version)"
latest_version="$(_get_latest_version)"

_info "Installed : ${installed_version}"
_info "Latest    : ${latest_version}"

if [[ "${installed_version}" != "none" && "${installed_version}" == "${latest_version}" ]]; then
  _ok "Already up to date (${installed_version})."
  exit 0
fi

_info "Upgrade available (${installed_version} -> ${latest_version}). Running installer..."

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${script_dir}/install.sh" ]]; then
  bash "${script_dir}/install.sh" "${latest_version}"
else
  curl -fsSL "https://raw.githubusercontent.com/${REPO}/main/scripts/install.sh" | bash -s -- "${latest_version}"
fi
