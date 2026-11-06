# Rust toolchain module: installs the pinned rustup into .tools/rust/, and through it the pinned Rust with clippy,
# rustfmt and rust-analyzer, plus a rust-analyzer launcher on PATH for the rust-analyzer-lsp plugin.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'
[private]
mod launcher '../launcher/mod.just'

# Install the pinned toolchain (a no-op when present), then the rust-analyzer launcher. The launcher runs a wrapper in
# .tools/ that sets rustup's and cargo's homes, which rust-analyzer needs to find the toolchain and its cargo.
install: (fetch::get "https://static.rust-lang.org/rustup/archive/" + rustup_version + "/" + rustup_target + "/rustup-init" rustup_sha256 rustup_init) && (launcher::write "rust-analyzer" rust_dir / "bin" / "rust-analyzer")
  #!/usr/bin/env bash
  set -euo pipefail
  export RUSTUP_HOME="{{ rustup_home }}" CARGO_HOME="{{ cargo_home }}"
  [ -x "{{ rustup }}" ] || "{{ rustup_init }}" -y --quiet --no-modify-path --default-toolchain none
  if ! "{{ rustup }}" which --toolchain "{{ rust_version }}" rust-analyzer >/dev/null 2>&1; then
    "{{ rustup }}" toolchain install --profile minimal --component clippy,rustfmt,rust-analyzer "{{ rust_version }}"
  fi
  "{{ rustup }}" default "{{ rust_version }}" >/dev/null 2>&1
  mkdir -p "{{ rust_dir }}/bin"
  printf '#!/bin/sh\nexport RUSTUP_HOME="%s" CARGO_HOME="%s" PATH="%s/bin:$PATH"\nexec "%s/bin/rust-analyzer" "$@"\n' \
    "$RUSTUP_HOME" "$CARGO_HOME" "$CARGO_HOME" "$CARGO_HOME" >"{{ rust_dir }}/bin/rust-analyzer"
  chmod +x "{{ rust_dir }}/bin/rust-analyzer"

# Report whether the pinned toolchain, its components and the rust-analyzer launcher on PATH are there
check:
  #!/usr/bin/env bash
  set -euo pipefail
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"; case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac # setup put it in the shell profiles; this shell may predate that
  v=$({{ rust_env }} "{{ cargo_home }}/bin/rustc" --version 2>/dev/null) || { echo "rust: missing (just rust install)"; exit 1; }
  [[ "$v" == "rustc {{ rust_version }} "* ]] || { echo "rust: '$v' installed, pinned {{ rust_version }} (just rust install)"; exit 1; }
  for c in clippy-driver rustfmt rust-analyzer; do
    {{ rust_env }} "{{ rustup }}" which "$c" >/dev/null 2>&1 || { echo "rust: $c missing (just rust install)"; exit 1; }
  done
  grep -qs '/\.tools/' "$(command -v rust-analyzer)" || { echo "rust: rust-analyzer on PATH is not the pinned one (just rust install)"; exit 1; }
  echo "rust: ok ({{ rust_version }} with clippy, rustfmt and rust-analyzer; rustup {{ rustup_version }})"
