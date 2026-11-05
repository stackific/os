# Self-hosted GitHub Actions runner settings. Runners need Linux: on a Mac the runner lives in a Linux VM (just runner vm).

# Pinned runner (github.com/actions/runner) and the SHA-256 of its release asset per Linux host. This pins the first
# install only: the runner then updates itself, because GitHub stops sending jobs to outdated runners.
runner_version := "2.338.2"
runner_target := "linux-" + (if arch() == "aarch64" { "arm64" } else { "x64" })
runner_sha256 := if runner_target == "linux-arm64" { "670adb2f268fe21dcbb58b3db0e0aa9d5b3bddfe2c651481336e561ff86de945" } else { "baebf204307bac6193480852732f0aba2ae97b6c5597c244f4d5558390147c40" }

# The repository the runner serves, and where the runner lives on this machine
runner_repo := "stackific/os"
runner_dir := justfile_directory() / ".tools" / "actions-runner"

# The Lima VM that hosts the runner on a Mac
runner_vm := "stackific-runner"
