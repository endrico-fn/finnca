#!/usr/bin/env bash
set -Eeuo pipefail
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -f "${script_dir}/finnca.sh" ]]; then
  exec bash "${script_dir}/finnca.sh" upgrade "$@"
else
  exec bash <(curl -fsSL "https://raw.githubusercontent.com/endrico-fn/finnca/main/scripts/finnca.sh") upgrade "$@"
fi
