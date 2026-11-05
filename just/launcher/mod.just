# Launcher module: puts a command on PATH (~/.local/bin) that runs a tool from .tools/.

import '../uv/vars.just'

[private]
mod uv '../uv/mod.just'

# Write launcher <name> running <target>; a file already there that does not point into .tools/ is the user's and is kept.
# When the launcher folder is not on PATH, the pinned uv adds it to the shell profiles (zsh, bash, fish; new shells).
write name target: uv::install
  #!/usr/bin/env bash
  set -euo pipefail
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"
  f="$d/{{ name }}"
  case ":$PATH:" in *":$d:"*) ;; *) UV_TOOL_BIN_DIR="$d" "{{ uv }}" tool update-shell ;; esac
  if { [ -e "$f" ] || [ -L "$f" ]; } && ! grep -qs '/\.tools/' "$f"; then
    echo "Keeping $f (not ours); remove it to use the pinned {{ name }}." >&2
    exit 0
  fi
  mkdir -p "$d"
  printf '#!/bin/sh\nexec "%s" "$@"\n' "{{ target }}" > "$f"
  chmod +x "$f"
