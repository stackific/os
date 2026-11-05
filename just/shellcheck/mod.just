# shellcheck module: installs the pinned shellcheck into .tools/shellcheck and lints shell files.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned shellcheck (no-op when present)
install: (fetch::get "https://github.com/koalaman/shellcheck/releases/download/v" + shellcheck_version + "/shellcheck-v" + shellcheck_version + "." + shellcheck_target + ".tar.gz" shellcheck_sha256 shellcheck "shellcheck-v" + shellcheck_version + "/shellcheck")

# Lint the given shell files (paths relative to the repo root; PKGBUILDs are bash), or every tracked one
[positional-arguments]
lint *files: install
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ justfile_directory() }}"
  if [ $# -eq 0 ]; then
    files=()
    while IFS= read -r -d '' f; do files+=("$f"); done < <(git ls-files -z -- '*.sh' '*.bash' '**/PKGBUILD' 'PKGBUILD')
    set -- ${files[@]+"${files[@]}"}
  fi
  [ $# -eq 0 ] || "{{ shellcheck }}" --shell=bash "$@"
