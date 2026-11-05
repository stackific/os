# Stackific OS: wires the modules and `setup`; each area's recipes live in its own module.

# Agent tools and the toolchains they run on
mod uv 'just/uv/mod.just'
mod rtk 'just/rtk/mod.just'
mod go 'just/go/mod.just'
mod serena 'just/serena/mod.just'
mod claude 'just/claude/mod.just'
mod graphify 'just/graphify/mod.just'

# Checks: git hooks (lefthook.yml) and the linters they run
mod lefthook 'just/lefthook/mod.just'
mod gitleaks 'just/gitleaks/mod.just'
mod shellcheck 'just/shellcheck/mod.just'
mod actionlint 'just/actionlint/mod.just'

# Task files, review records and their prompt-trail entries (recipes in dev/justfile)
mod dev

# Arch base: live ISO build and boot test in the pinned builder container (recipes in arch/justfile)
mod arch

# Website: marketing pages at / and Starlight docs at /docs (recipes in site/justfile)
mod site

# List every recipe
default:
  @just --list --list-submodules

# graphify comes last among the agent tools: its project install rewrites .claude/settings.json after the plugin installs.
# Install every tool this repo needs (host prerequisites: README.md), then run doctor
setup: uv::install rtk::install go::install serena::install claude::plugins graphify::install gitleaks::install shellcheck::install actionlint::install site::install lefthook::hooks doctor

# Check every tool's pinned version and wiring, and the host prerequisites setup cannot install
doctor:
  #!/usr/bin/env bash
  set -uo pipefail
  rc=0
  for m in uv rtk go serena claude graphify lefthook arch; do "{{ just_executable() }}" --quiet "$m::check" || rc=1; done
  exit "$rc"
