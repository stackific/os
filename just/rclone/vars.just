# Shared rclone settings. Import this from any module that runs rclone.

# Pinned rclone (github.com/rclone/rclone, checked 2026-11-08; v1.76.0 was two days old) and the SHA-256 of its release
# asset per host, from the release's SHA256SUMS
rclone_version := "1.75.2"
rclone_target := (if os() == "macos" { "osx" } else { "linux" }) + "-" + (if arch() == "aarch64" { "arm64" } else { "amd64" })
rclone_sha256 := if rclone_target == "osx-arm64" { "610ab7d48615135dfc60b763858b11a426a0d3d1160c1bb76fb0ae6011b02edd" } else if rclone_target == "linux-arm64" { "7e1e8d69654941b7b7df84ee74c5f7fb09cce7d5496947fb7bf16b55ff427d10" } else { "349ac8fba6ff65d6247043f1750cdcb518ec5d500ef91463a10d37c0ccdf3702" }

rclone := justfile_directory() / ".tools" / "rclone" / "rclone"
