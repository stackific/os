#!/usr/bin/env bash
# kiwi's customize script for the Terra repository (config.kiwi): kiwi writes the repo file, then runs this script with
# the file's path as $1. kiwi has no attribute for either line, so they are appended here.
# includepkgs: Terra supplies only these packages. Unlimited, dnf takes Terra's cardwire (a GPU manager) for
# gnome-shell's switcheroo-control instead of Fedora's, and Terra's builds of packages Fedora also ships.
# gpgkey: dnf checks Terra's repository metadata signature (repository_gpgcheck) only against a key named here; the
# package signature check uses the same key, imported into the image's rpmdb from config.kiwi's <signing>.
set -euo pipefail

printf 'includepkgs=mangowm,noctalia-greeter,limine\ngpgkey=file:///desc/terra-45.asc\n' >>"$1"
