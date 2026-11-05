# Shared gh settings. Import this from any module that runs gh.

# Pinned GitHub CLI (github.com/cli/cli) and the SHA-256 of its release asset per host
gh_version := "2.103.3"
gh_target := if os() == "macos" { "macOS_arm64" } else if arch() == "aarch64" { "linux_arm64" } else { "linux_amd64" }
gh_archive := "gh_" + gh_version + "_" + gh_target + (if os() == "macos" { ".zip" } else { ".tar.gz" })
gh_sha256 := if gh_target == "macOS_arm64" { "d700207eae9ae98fe2b89535c2c087c07823d4628c56364695a5a07ef311c062" } else if gh_target == "linux_arm64" { "ee32df2323c8847f5fac6cb6b4b69ca75828e9329a8f22f677021f7b3306ce63" } else { "4c806978ffce1cec70853302a56d70ac6da4f19bae7adca4e1a8ab00633dd91d" }

gh := justfile_directory() / ".tools" / "gh" / "gh"
