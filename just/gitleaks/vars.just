# Shared gitleaks settings. Import this from any module that runs gitleaks.

# Pinned gitleaks (github.com/gitleaks/gitleaks) and the SHA-256 of its release asset per host
gitleaks_version := "8.30.1"
gitleaks_target := (if os() == "macos" { "darwin" } else { "linux" }) + "_" + (if arch() == "aarch64" { "arm64" } else { "x64" })
gitleaks_sha256 := if gitleaks_target == "darwin_arm64" { "b40ab0ae55c505963e365f271a8d3846efbc170aa17f2607f13df610a9aeb6a5" } else if gitleaks_target == "linux_arm64" { "e4a487ee7ccd7d3a7f7ec08657610aa3606637dab924210b3aee62570fb4b080" } else { "551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb" }

gitleaks := justfile_directory() / ".tools" / "gitleaks" / "gitleaks"
