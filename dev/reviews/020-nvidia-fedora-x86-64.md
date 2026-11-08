# 020: NVIDIA on Fedora x86_64: open modules built into the image, the driver picked at every boot, nouveau for older cards: review

Task: dev/tasks/020-nvidia-fedora-x86-64.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (the generator's missing-module rule and its test sets, the kmod build in `just fedora packages`, the RPM Fusion repositories and packages in `config.kiwi`, `stackific-hardware`'s RPM and its two Fedora-only files, the live user's MangoWM config, the installer's `grubby` removal, the checks and docs).
Verdict: Looks good. Ship. (Checked and dropped before reporting: whether `rpmkeys -K` passes an unsigned package after `dnf5 download`, which checks no signature. rpm 6.1 in the builder prints `digests SIGNATURES NOT OK` and exits 1 for one, so the plain call stops the build; a comment says so. Also checked: the generator's `find` stops at the first `nvidia.ko*` and does not follow the dangling `build` link; a newer Fedora kernel published between `packages` and `iso` leaves the image's kernel without a module, which the boot test's vermagic check catches and the docs explain.)

Tests after the fixes: `just hardware test` (13 sets), `just installer lint`, `just installer test`, `just fedora packages`, `just fedora iso` (1921 MiB), `just fedora test`, `just fedora test-install`, `just arch packages` and `just lefthook all` pass.
