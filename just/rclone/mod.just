# rclone module: installs the pinned rclone into .tools/rclone, which `just release upload` uses for an ISO over 2 GiB
# (Cloudflare R2).

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned rclone (no-op when present)
install: (fetch::get "https://github.com/rclone/rclone/releases/download/v" + rclone_version + "/rclone-v" + rclone_version + "-" + rclone_target + ".zip" rclone_sha256 rclone "rclone-v" + rclone_version + "-" + rclone_target + "/rclone")

# Check the installed rclone is the pinned one
check:
  #!/usr/bin/env bash
  set -euo pipefail
  v=$("{{ rclone }}" version 2>/dev/null | head -1) || { echo "rclone: missing (just rclone install)"; exit 1; }
  [ "$v" = "rclone v{{ rclone_version }}" ] || { echo "rclone: '$v', pinned {{ rclone_version }} (just rclone install)"; exit 1; }
  echo "rclone: ok ({{ rclone_version }})"
