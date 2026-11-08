# Serena module: installs the pinned Serena as a uv tool with a launcher on PATH, which .mcp.json runs directly, so
# Claude Code starts it in any checkout or worktree even when no justfile parses (`just` stops at a missing module).
# Its config for this repo is .serena/project.yml; local overrides go in the untracked .serena/project.local.yml.

import '../uv/vars.just'
import 'vars.just'

[private]
mod uv '../uv/mod.just'
[private]
mod launcher '../launcher/mod.just'

# Install the pinned Serena (no-op when present) and its launcher
install: uv::install (launcher::write "serena" tools_bin / "serena" serena_env)
  @{{ uv_tool_install }} serena-agent=={{ serena_version }}

# Start this repo's language servers and query them end to end (slow; run after setup or a language change)
health:
  {{ serena_env }} "{{ tools_bin }}/serena" project health-check "{{ justfile_directory() }}"

# Report whether the pinned Serena is installed, its launcher is the `serena` on PATH (what .mcp.json runs), and npm
# is there for its Bash language server. The version comes from uv: `serena --version` appends the current directory's
# git commit.
check:
  #!/usr/bin/env bash
  set -euo pipefail
  {{ uv_env }} "{{ uv }}" tool list 2>/dev/null | grep -qx "serena-agent v{{ serena_version }}" \
    || { echo "serena: pinned {{ serena_version }} missing (just serena install)"; exit 1; }
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"; case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac # setup put it in the shell profiles; this shell may predate that
  f=$(command -v serena) || { echo "serena: not on PATH, so Claude Code cannot start it (just serena install; put ~/.local/bin on PATH)"; exit 1; }
  grep -qs '/\.tools/' "$f" || { echo "serena: $f is not this repo's launcher, so Claude Code runs another Serena (remove it, then just serena install)"; exit 1; }
  grep -qsF "export {{ serena_env }}" "$f" || { echo "serena: $f does not set SERENA_HOME, so Serena reads ~/.serena (just serena install)"; exit 1; }
  command -v npm >/dev/null || { echo "serena: npm missing; Serena installs its Bash language server with it (install nodejs and npm)"; exit 1; }
  echo "serena: ok ({{ serena_version }})"
