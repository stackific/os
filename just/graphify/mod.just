# graphify module: installs the pinned graphify as a uv tool with a launcher on PATH (its Claude Code hooks call it),
# adds its skill and hooks at project scope, and rebuilds code graphs.

import '../uv/vars.just'
import 'vars.just'

[private]
mod uv '../uv/mod.just'
[private]
mod launcher '../launcher/mod.just'

# The hooks and the CLAUDE.md section are committed; re-running this after a pin bump shows their diff.
# Install the pinned graphify and its launcher, then its project-scope skill, hooks and CLAUDE.md section
install: uv::install (launcher::write "graphify" tools_bin / "graphify")
  @{{ uv_tool_install }} graphifyy=={{ graphify_version }}
  @cd "{{ justfile_directory() }}" && "{{ tools_bin }}/graphify" install --project > /dev/null

# Rebuild the code graph (AST only, no LLM) of the repo, or of <dir> (relative to the repo root), into its graphify-out/
update dir=".":
  cd "{{ justfile_directory() }}" && "{{ tools_bin }}/graphify" update "{{ dir }}"

# Report whether graphify on PATH is the pinned one and its project skill is installed
check:
  #!/usr/bin/env bash
  set -euo pipefail
  v=$(graphify --version 2>/dev/null) || { echo "graphify: not on PATH (just graphify install; put ~/.local/bin on PATH)"; exit 1; }
  [ "$v" = "graphify {{ graphify_version }}" ] || { echo "graphify: $(command -v graphify) is '$v', pinned {{ graphify_version }}"; exit 1; }
  [ -f "{{ justfile_directory() }}/.claude/skills/graphify/SKILL.md" ] || { echo "graphify: project skill missing (just graphify install)"; exit 1; }
  echo "graphify: ok ({{ graphify_version }})"
