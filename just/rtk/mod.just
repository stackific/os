# rtk module: installs the pinned rtk into .tools/rtk with a launcher on PATH, where the Claude Code Bash hook
# (`rtk hook claude` in .claude/settings.json) finds it.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'
[private]
mod launcher '../launcher/mod.just'

# Install the pinned rtk (no-op when present) and its launcher
install: (fetch::get "https://github.com/rtk-ai/rtk/releases/download/v" + rtk_version + "/rtk-" + rtk_target + ".tar.gz" rtk_sha256 rtk_dir / "rtk" "rtk") (launcher::write "rtk" rtk_dir / "rtk")

# Report whether rtk on PATH is the pinned one and the Bash hook rewrites commands
check:
  #!/usr/bin/env bash
  set -euo pipefail
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"; case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac # setup put it in the shell profiles; this shell may predate that
  v=$(rtk --version 2>/dev/null) || { echo "rtk: not on PATH (just rtk install; put ~/.local/bin on PATH)"; exit 1; }
  [ "$v" = "rtk {{ rtk_version }}" ] || { echo "rtk: $(command -v rtk) is '$v', pinned {{ rtk_version }} (put ~/.local/bin ahead of it on PATH)"; exit 1; }
  grep -qF '"rtk hook claude"' "{{ justfile_directory() }}/.claude/settings.json" \
    || { echo "rtk: Bash hook missing from .claude/settings.json"; exit 1; }
  echo '{"tool_name":"Bash","tool_input":{"command":"git status"}}' | rtk hook claude | grep -qF '"rtk git status"' \
    || { echo "rtk: the hook does not rewrite commands"; exit 1; }
  echo "rtk: ok ({{ rtk_version }}, Bash hook rewrites)"
