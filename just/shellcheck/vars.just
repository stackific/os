# Shared shellcheck settings. Import this from any module that runs shellcheck.

# Pinned shellcheck (github.com/koalaman/shellcheck, which publishes no checksums: these are of the downloads, 2026-11-04)
shellcheck_version := "0.11.0"
shellcheck_target := (if os() == "macos" { "darwin" } else { "linux" }) + "." + (if arch() == "aarch64" { "aarch64" } else { "x86_64" })
shellcheck_sha256 := if shellcheck_target == "darwin.aarch64" { "339b930feb1ea764467013cc1f72d09cd6b869ebf1013296ba9055ab2ffbd26f" } else if shellcheck_target == "linux.aarch64" { "68a8133197a50beb8803f8d42f9908d1af1c5540d4bb05fdfca8c1fa47decefc" } else { "b7af85e41cc99489dcc21d66c6d5f3685138f06d34651e6d34b42ec6d54fe6f6" }

shellcheck := justfile_directory() / ".tools" / "shellcheck" / "shellcheck"
