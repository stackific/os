# Runner module: makes this machine a self-hosted GitHub Actions runner for the repo's image jobs (images workflow).
# Only for the machines that serve CI: the operator's Linux PC (x86_64) and the Linux VM on the Mac (aarch64).
# It is not part of `just setup`, because registering hands the machine to GitHub jobs.

import 'vars.just'
import '../host/vars.just'
import '../gh/vars.just'

[private]
mod gh '../gh/mod.just'

# Make this machine a runner: install, register with GitHub, then run as a service (asks for sudo once)
setup: install register service

# Install the pinned runner into .tools/actions-runner (no-op when present)
install:
  #!/usr/bin/env bash
  set -euo pipefail
  [ "{{ os() }}" = linux ] || { echo "runner: needs Linux; on a Mac, run it in the VM from 'just runner vm'" >&2; exit 1; }
  [ "$(cat "{{ runner_dir }}/.pin" 2>/dev/null)" != "{{ runner_sha256 }}" ] || exit 0
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  f="$tmp/runner.tar.gz"
  curl -fsSLo "$f" "https://github.com/actions/runner/releases/download/v{{ runner_version }}/actions-runner-{{ runner_target }}-{{ runner_version }}.tar.gz"
  echo "{{ runner_sha256 }}  $f" | {{ sha256_check }}
  mkdir -p "{{ runner_dir }}"
  tar -xzf "$f" -C "{{ runner_dir }}"
  echo "{{ runner_sha256 }}" > "{{ runner_dir }}/.pin"

# It also makes every outside contributor's workflow run wait for approval, so code from a fork never runs here
# unreviewed. Labels: self-hosted, Linux and X64 or ARM64.
# Register this machine as a runner for the repo
register: install gh::login
  #!/usr/bin/env bash
  set -euo pipefail
  "{{ gh }}" api --silent -X PUT "repos/{{ runner_repo }}/actions/permissions/fork-pr-contributor-approval" \
    -f approval_policy=all_external_contributors
  token=$("{{ gh }}" api -X POST "repos/{{ runner_repo }}/actions/runners/registration-token" --jq .token)
  cd "{{ runner_dir }}"
  ./config.sh --unattended --replace --url "https://github.com/{{ runner_repo }}" --token "$token" --name "$(hostname)"

# Run the registered runner as a systemd service under this user, started at boot (the runner's own svc.sh, via sudo)
service:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ runner_dir }}"
  [ -f .runner ] || { echo "runner: not registered (just runner register)" >&2; exit 1; }
  # Rootless podman keeps its runtime directory without a login session
  loginctl enable-linger "$USER"
  [ -f .service ] || sudo ./svc.sh install "$USER"
  sudo ./svc.sh start

# List the repo's runners: name, status (online or offline), busy, labels
status: gh::login
  @"{{ gh }}" api "repos/{{ runner_repo }}/actions/runners" --jq '.runners[] | "\(.name) \(.status) busy=\(.busy) \([.labels[].name] | join(","))"'

# Stop the service and unregister this machine
remove: gh::login
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ runner_dir }}"
  if [ -f .service ]; then sudo ./svc.sh stop; sudo ./svc.sh uninstall; fi
  token=$("{{ gh }}" api -X POST "repos/{{ runner_repo }}/actions/runners/remove-token" --jq .token)
  ./config.sh remove --token "$token"

# Lima with nested virtualization needs Apple M3 or later and macOS 15 or later. The VM mounts nothing from the Mac,
# so jobs cannot see its files.
# On a Mac: create or start the arm64 Fedora VM that hosts the runner
vm:
  #!/usr/bin/env bash
  set -euo pipefail
  command -v limactl >/dev/null || { echo "runner: limactl missing (brew install lima)" >&2; exit 1; }
  if limactl list --quiet | grep -qx "{{ runner_vm }}"; then
    limactl start --tty=false "{{ runner_vm }}"
  else
    limactl start --tty=false --name="{{ runner_vm }}" --vm-type=vz --nested-virt --mount-none \
      --cpus=6 --memory=12 --disk=100 template:fedora
  fi
  echo "runner: VM up; open a shell in it with: limactl shell {{ runner_vm }}"
