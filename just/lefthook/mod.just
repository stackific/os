# lefthook module: installs the pinned lefthook, wires the git hooks to just recipes (lefthook.yml at the repo root)
# and holds the repo-wide checks those hooks run.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned lefthook (no-op when present)
install: (fetch::get "https://github.com/evilmartians/lefthook/releases/download/v" + lefthook_version + "/" + lefthook_asset lefthook_sha256 lefthook)

# Install lefthook, then (re)write .git/hooks from lefthook.yml
hooks: install
  @cd "{{ justfile_directory() }}" && "{{ lefthook }}" install

# Run every pre-commit check on every tracked file (used by the check workflow)
all: install
  @cd "{{ justfile_directory() }}" && "{{ lefthook }}" run pre-commit --all-files

# Fail on whitespace errors, conflict markers, files over 1 MiB (wallpapers excepted) or names of local untracked
# material in the given files, relative to the repo root (used by the pre-commit hook)
[positional-arguments]
hygiene *files:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ justfile_directory() }}"
  # Local untracked material is listed in .git/info/exclude as root paths (`/name/`); tracked files must not name it
  names=$(sed -n 's|^/\([^*?[]*[^/]\)/*$|\1|p' "$(git rev-parse --git-path info/exclude)" 2>/dev/null || true)
  rc=0
  for f in "$@"; do
    [ -f "$f" ] || continue
    out=$(git diff --no-index --check /dev/null "$f" || true)
    if [ -n "$out" ]; then echo "$out" >&2; rc=1; fi
    if [ "$(wc -c < "$f")" -gt 1048576 ] && [[ "$f" != branding/wallpapers/* ]]; then
      echo "$f: over 1 MiB; keep large files out of git" >&2; rc=1
    fi
    if [ -n "$names" ] && grep -qF "$names" -- "$f"; then
      echo "$f: names local untracked material listed in .git/info/exclude" >&2; rc=1
    fi
  done
  exit "$rc"

# Fail unless every justfile and module parses (used by the pre-commit hook)
justfiles:
  @cd "{{ justfile_directory() }}" && just --list --list-submodules > /dev/null

# Fail unless the commit message file carries a DCO `Signed-off-by:` trailer (used by the commit-msg hook);
# [no-cd]: git passes a path relative to the repo root
[no-cd]
dco msg_file:
  @grep -q '^Signed-off-by: ' "{{ msg_file }}" || { echo "Missing DCO sign-off: commit with 'git commit -s'." >&2; exit 1; }

# Report whether the git hooks are installed
check:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ justfile_directory() }}"
  for h in pre-commit commit-msg; do
    grep -qs lefthook "$(git rev-parse --git-path "hooks/$h")" || { echo "lefthook: $h hook missing (just lefthook hooks)"; exit 1; }
  done
  echo "lefthook: ok ({{ lefthook_version }}, hooks installed)"
