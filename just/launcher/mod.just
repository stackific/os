# Launcher module: puts a command on PATH (~/.local/bin) that runs a tool from .tools/.

# Write launcher <name> running <target>; a file already there that does not point into .tools/ is the user's and is kept
write name target:
  #!/usr/bin/env bash
  set -euo pipefail
  f="${XDG_BIN_HOME:-$HOME/.local/bin}/{{ name }}"
  if { [ -e "$f" ] || [ -L "$f" ]; } && ! grep -qs '/\.tools/' "$f"; then
    echo "Keeping $f (not ours); remove it to use the pinned {{ name }}." >&2
    exit 0
  fi
  mkdir -p "$(dirname "$f")"
  printf '#!/bin/sh\nexec "%s" "$@"\n' "{{ target }}" > "$f"
  chmod +x "$f"
