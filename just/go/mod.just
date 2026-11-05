# Go module: installs the pinned Go and gopls into .tools/, so Serena can read Go sources.
# Not on PATH: `just serena mcp` puts both first on Serena's PATH.

import '../host/vars.just'
import 'vars.just'

# Install the pinned Go and gopls (each a no-op when present)
install:
  #!/usr/bin/env bash
  set -euo pipefail
  if [ "$("{{ go_dir }}/bin/go" env GOVERSION 2>/dev/null)" != "go{{ go_version }}" ]; then
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    curl -fsSLo "$tmp/go.tar.gz" "https://go.dev/dl/go{{ go_version }}.{{ go_target }}.tar.gz"
    echo "{{ go_sha256 }}  $tmp/go.tar.gz" | {{ sha256_check }}
    rm -rf "{{ go_dir }}" && mkdir -p "{{ go_dir }}"
    tar -xzf "$tmp/go.tar.gz" -C "{{ go_dir }}" --strip-components=1
  fi
  # A Go bump rebuilds gopls too: gopls type-checks with the Go it was built with
  if ! { "{{ gopls_dir }}/bin/gopls" version 2>/dev/null | grep -qF "golang.org/x/tools/gopls {{ gopls_version }}" \
    && [[ "$("{{ go_dir }}/bin/go" version "{{ gopls_dir }}/bin/gopls" 2>/dev/null)" == *": go{{ go_version }}" ]]; }; then
    GOTOOLCHAIN=local GOBIN="{{ gopls_dir }}/bin" "{{ go_dir }}/bin/go" install "golang.org/x/tools/gopls@{{ gopls_version }}"
  fi

# Report whether the pinned Go and gopls are installed
check:
  #!/usr/bin/env bash
  set -euo pipefail
  [ "$("{{ go_dir }}/bin/go" env GOVERSION 2>/dev/null)" = "go{{ go_version }}" ] || { echo "go: pinned {{ go_version }} missing (just go install)"; exit 1; }
  "{{ gopls_dir }}/bin/gopls" version 2>/dev/null | grep -qF "golang.org/x/tools/gopls {{ gopls_version }}" \
    || { echo "go: gopls {{ gopls_version }} missing (just go install)"; exit 1; }
  echo "go: ok (go {{ go_version }}, gopls {{ gopls_version }})"
