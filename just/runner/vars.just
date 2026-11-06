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

# Its OS: Fedora Cloud for aarch64, pinned by the SHA-256 from Fedora's signed CHECKSUM file. The VM's Fedora release
# must be the Fedora images' release: kiwi labels the image as rootless container root, which may set only SELinux
# contexts the VM's own policy knows. Lima's template:fedora follows Lima's release, not ours. `just doctor` checks it.
# The path is under /pub/ on Fedora's mirror redirector, then on Fedora's own server: Lima tries them in order, because
# the redirector sometimes picks a dead mirror.
runner_vm_fedora := "45"
runner_vm_image := "fedora/linux/releases/" + runner_vm_fedora + "/Cloud/aarch64/images/Fedora-Cloud-Base-Generic-" + runner_vm_fedora + "-1.4.aarch64.qcow2"
runner_vm_digest := "sha256:1632acc8bc14d45b6c9596a0acc71b070d3748d1abdcf1a519be5c983bb0286c"
