# Serena module: installs the pinned Serena as a uv tool and serves it over MCP for Claude Code (.mcp.json).
# Its config for this repo is .serena/project.yml; local overrides go in the untracked .serena/project.local.yml.

import '../uv/vars.just'
import '../go/vars.just'
import 'vars.just'

[private]
mod uv '../uv/mod.just'

# Install the pinned Serena (no-op when present)
install: uv::install
  @{{ uv_tool_install }} serena-agent=={{ serena_version }}

# Serena's Go language server: the pinned Go and gopls come first on PATH in every recipe here
export PATH := go_dir / "bin" + ":" + gopls_dir / "bin" + ":" + env("PATH")

# Serve Serena over MCP on stdio for this repo (launched by .mcp.json)
mcp:
  @exec "{{ tools_bin }}/serena" start-mcp-server --context claude-code --project "{{ justfile_directory() }}"

# Start this repo's language servers and query them end to end (slow; run after setup or a language change)
health:
  "{{ tools_bin }}/serena" project health-check "{{ justfile_directory() }}"

# Report whether the pinned Serena is installed and npm is there for its Bash language server.
# The version comes from uv: `serena --version` appends the current directory's git commit.
check:
  #!/usr/bin/env bash
  set -euo pipefail
  {{ uv_env }} "{{ uv }}" tool list 2>/dev/null | grep -qx "serena-agent v{{ serena_version }}" \
    || { echo "serena: pinned {{ serena_version }} missing (just serena install)"; exit 1; }
  command -v npm >/dev/null || { echo "serena: npm missing; Serena installs its Bash language server with it (install nodejs and npm)"; exit 1; }
  echo "serena: ok ({{ serena_version }})"
