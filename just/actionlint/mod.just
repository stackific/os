# actionlint module: installs the pinned actionlint into .tools/actionlint and lints the GitHub Actions workflows.

import 'vars.just'
import '../shellcheck/vars.just'

[private]
mod fetch '../fetch/mod.just'
[private]
mod shellcheck '../shellcheck/mod.just'

# Install the pinned actionlint (no-op when present)
install: (fetch::get "https://github.com/rhysd/actionlint/releases/download/v" + actionlint_version + "/actionlint_" + actionlint_version + "_" + actionlint_target + ".tar.gz" actionlint_sha256 actionlint "actionlint")

# Lint every workflow in .github/workflows/, with the pinned shellcheck on their run: scripts
lint: install shellcheck::install
  @cd "{{ justfile_directory() }}" && "{{ actionlint }}" -shellcheck "{{ shellcheck }}"
