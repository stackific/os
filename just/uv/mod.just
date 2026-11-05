# uv toolchain module: installs the pinned uv into .tools/uv.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned uv (no-op when present); uv fetches its Python on first use
install: (fetch::get "https://github.com/astral-sh/uv/releases/download/" + uv_version + "/uv-" + uv_target + ".tar.gz" uv_sha256 uv "uv-" + uv_target + "/uv")

# Report whether the pinned uv is installed
check:
  #!/usr/bin/env bash
  set -euo pipefail
  v=$("{{ uv }}" --version 2>/dev/null) || { echo "uv: missing (just uv install)"; exit 1; }
  [[ "$v" == "uv {{ uv_version }} "* ]] || { echo "uv: '$v' installed, pinned {{ uv_version }} (just uv install)"; exit 1; }
  echo "uv: ok ({{ uv_version }})"
