# gh module: installs the pinned GitHub CLI into .tools/gh/gh (not on PATH; the runner module drives it).

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned gh (no-op when present)
install: (fetch::get "https://github.com/cli/cli/releases/download/v" + gh_version + "/" + gh_archive gh_sha256 gh "gh_" + gh_version + "_" + gh_target + "/bin/gh")

# Log gh in to GitHub, once per machine (no-op when already logged in)
login: install
  @"{{ gh }}" auth status >/dev/null 2>&1 || "{{ gh }}" auth login
