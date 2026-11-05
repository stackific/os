# Shared Go settings. Import this from any module that runs Go or gopls.

# Pinned Go (go.dev/dl) and the SHA-256 of its archive per host
go_version := "1.27.3"
go_target := (if os() == "macos" { "darwin" } else { "linux" }) + "-" + (if arch() == "aarch64" { "arm64" } else { "amd64" })
go_sha256 := if go_target == "darwin-arm64" { "dd3908cf20959d798c1e06ebe5b72213f0505ce01373b40ce3110975a6cd99c2" } else if go_target == "linux-arm64" { "a06e99309dbcbdca1a8ccd2282c3ee146e55930886e26b4f1193269587165f49" } else { "65efbc80fdc1fa2cd2631cad5b481777a68e8725f34e89f50fc6ac714e379b40" }

# Pinned gopls, built with the pinned Go (`go install` checks it against the Go checksum database)
gopls_version := "v0.24.0"

go_dir := justfile_directory() / ".tools" / "go"
gopls_dir := justfile_directory() / ".tools" / "gopls"
