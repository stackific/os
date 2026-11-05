# Shared Bun settings. Import this from any module that runs Bun.

# Pinned Bun (github.com/oven-sh/bun) and the SHA-256 of its release asset per host
bun_version := "1.4.4"
bun_target := (if os() == "macos" { "darwin" } else { "linux" }) + "-" + (if arch() == "aarch64" { "aarch64" } else { "x64" })
bun_sha256 := if bun_target == "darwin-aarch64" { "c240d7f0ccb90c263a849fcdb09b7ee62630d921348ed6aafda86b9d577bc510" } else if bun_target == "linux-aarch64" { "baa6bb36f51d6e58561e02bbea825038a9df454f673d6d776e8bbf86f8c7850e" } else { "bf8ccbfb2b0b80159e0f9a6d15c55317add8077534dbc6130a3edb52fccb395c" }

bun_bin := justfile_directory() / ".tools" / "bun" / "bun"

# Bun command; `--bun` runs node-shebang CLIs (astro, eslint, prettier) on Bun, so Node is never needed
bun := "'" + bun_bin + "' --bun"
