# 021: Regular hardware on both x86_64 images: matched firmware, Thunderbolt, fingerprint readers and MIPI webcams: review

Task: dev/tasks/021-hardware-parity.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (both images' package lists, Arch's `NoExtract`, the rclone module, `just release upload`'s R2 branch and the workflows' upload environments, the boot checks, docs), after two checks the test runs had found wrong were fixed: the libcamera plugin check (Arch's `/usr/lib64` links to `/usr/lib`, so the glob matched twice) and the PAM check (kscreenlocker's own `kde-fingerprint` names `pam_fprintd`).
Verdict: Looks good. Ship. (Checked: an empty secret fails `${…:?}` before any upload; a malformed `R2_ENDPOINT` stops with its expected form; the keys never reach a command line; a failed rclone copy stops before `gh release upload`, shown by a local run with a 2.1 GB sparse ISO against a closed port. Each merged pull request's test ISO stays in R2 as its test release stays on GitHub: the same retention, not a new problem.)

Tests after the fixes: `just arch iso` (2069 MiB), `just arch test`, `just arch test-install`, `just fedora iso`, `just fedora test`, `just fedora test-install`, `just rclone check`, `just lefthook all` pass.
