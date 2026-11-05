# gitleaks module: installs the pinned gitleaks into .tools/gitleaks and scans for secrets.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned gitleaks (no-op when present)
install: (fetch::get "https://github.com/gitleaks/gitleaks/releases/download/v" + gitleaks_version + "/gitleaks_" + gitleaks_version + "_" + gitleaks_target + ".tar.gz" gitleaks_sha256 gitleaks "gitleaks")

# Fail if the staged changes hold a secret (used by the pre-commit hook)
staged: install
  @cd "{{ justfile_directory() }}" && "{{ gitleaks }}" git --pre-commit --staged --redact --no-banner

# Fail if any commit in the history holds a secret (used by the check workflow)
history: install
  @cd "{{ justfile_directory() }}" && "{{ gitleaks }}" git --redact --no-banner
