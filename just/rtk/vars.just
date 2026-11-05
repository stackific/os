# Shared rtk settings. Import this from any module that runs rtk.

# Pinned rtk release (github.com/rtk-ai/rtk) and the SHA-256 of its release asset per host
rtk_version := "1.1.0"
rtk_target := if os() == "macos" { arch() + "-apple-darwin" } else if arch() == "aarch64" { "aarch64-unknown-linux-gnu" } else { "x86_64-unknown-linux-musl" }
rtk_sha256 := if rtk_target == "aarch64-apple-darwin" { "8da773b0954cf90b115ec95c4e4eb420f04c39de233c5db102246a6caf073ba7" } else if rtk_target == "aarch64-unknown-linux-gnu" { "a5c00c3ab0129884bcb757d348afc1a650efa0f4f0769da97c341f28f89719ed" } else { "9454ad1b9489daa57ab0a9f4ebe2faaf4dc82b8a0f26c9e5135151abd7c9378d" }

rtk_dir := justfile_directory() / ".tools" / "rtk"
