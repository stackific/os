# Stackific OS: wires the modules and `setup`; each area's recipes live in its own module.

# Agent tools and the toolchains they run on
mod uv 'just/uv/mod.just'
mod rtk 'just/rtk/mod.just'
mod serena 'just/serena/mod.just'
mod claude 'just/claude/mod.just'
mod graphify 'just/graphify/mod.just'

# Rust: the pinned toolchain the installer builds with
mod rust 'just/rust/mod.just'

# Checks: git hooks (lefthook.yml) and the linters they run
mod lefthook 'just/lefthook/mod.just'
mod gitleaks 'just/gitleaks/mod.just'
mod shellcheck 'just/shellcheck/mod.just'
mod actionlint 'just/actionlint/mod.just'

# GitHub: the gh CLI, and the self-hosted runner for the images workflow (`just runner setup`, CI machines only), whose
# Mac VM `just setup` also makes for image builds
mod gh 'just/gh/mod.just'
mod runner 'just/runner/mod.just'

# Task files, review records and their prompt-trail entries (recipes in dev/justfile)
mod dev

# Arch base: live ISO build and boot test in the pinned builder container (recipes in arch/justfile)
mod arch

# The installer: one Rust program for both bases (recipes in installer/justfile)
mod installer

# Fedora base: live ISO build and boot test in the pinned builder container (recipes in fedora/justfile)
mod fedora

# GitHub Releases for the images: draft, upload, publish (recipes in release/justfile; release and images workflows)
mod release

# Website: marketing pages at / and Starlight docs at /docs (recipes in site/justfile)
mod site

# List every recipe
default:
  @just --list --list-submodules

# graphify comes last among the agent tools: its project install rewrites .claude/settings.json after the plugin installs.
# runner::vm makes or starts the Mac's Fedora VM for image builds (a no-op on Linux); it registers no runner.
# Install every tool this repo needs (host prerequisites: docs/setup.md), then run doctor
setup: uv::install rtk::install serena::install claude::plugins graphify::install rust::install gitleaks::install shellcheck::install actionlint::install gh::install site::install lefthook::hooks runner::vm doctor

# Check every tool's pinned version and wiring, and the host prerequisites setup cannot install
doctor:
  #!/usr/bin/env bash
  set -uo pipefail
  rc=0
  for m in uv rtk serena claude graphify rust lefthook arch fedora runner; do "{{ just_executable() }}" --quiet "$m::check" || rc=1; done
  exit "$rc"
