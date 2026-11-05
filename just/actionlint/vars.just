# Shared actionlint settings. Import this from any module that runs actionlint.

# Pinned actionlint (github.com/rhysd/actionlint) and the SHA-256 of its release asset per host
actionlint_version := "1.7.12"
actionlint_target := (if os() == "macos" { "darwin" } else { "linux" }) + "_" + (if arch() == "aarch64" { "arm64" } else { "amd64" })
actionlint_sha256 := if actionlint_target == "darwin_arm64" { "aba9ced2dee8d27fecca3dc7feb1a7f9a52caefa1eb46f3271ea66b6e0e6953f" } else if actionlint_target == "linux_arm64" { "325e971b6ba9bfa504672e29be93c24981eeb1c07576d730e9f7c8805afff0c6" } else { "8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8" }

actionlint := justfile_directory() / ".tools" / "actionlint" / "actionlint"
