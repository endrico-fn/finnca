#!/usr/bin/env bash
set -Eeuo pipefail

script_source="${BASH_SOURCE[0]:-}"
if [[ -n "${script_source}" && -f "${script_source}" ]]; then
  script_dir="$(cd "$(dirname "${script_source}")" && pwd)"
  if [[ -f "${script_dir}/finnca.sh" ]]; then
    exec bash "${script_dir}/finnca.sh" install "$@"
  fi
fi

exec bash <(curl -fsSL "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.sh") install "$@"
