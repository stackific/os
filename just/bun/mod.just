# Bun toolchain module: installs the pinned Bun into .tools/bun.

import 'vars.just'

[private]
mod fetch '../fetch/mod.just'

# Install the pinned Bun (no-op when present)
install: (fetch::get "https://github.com/oven-sh/bun/releases/download/bun-v" + bun_version + "/bun-" + bun_target + ".zip" bun_sha256 bun_bin "bun-" + bun_target + "/bun")
