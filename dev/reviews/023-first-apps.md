# 023: First apps on all four images: review

Task: dev/tasks/023-first-apps.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (package lists, `config.kiwi`, `config.sh`, `stackific-desktop.spec` with `flathub.flatpakrepo`, `fedora/justfile`, the boot and install checks, docs, the user page and the entry).
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | Bazaar's runtime adds about 1.9 GiB to kiwi's tmpfs peak; on the Mac's 16 GiB VM the default half-RAM cap (8 GiB) is below the aarch64 build's new peak of about 8.5 GiB, so the build would stop with `No space left on device` | `fedora/justfile:96` | fixed | Task 022, merged to `main` first, already mounts the tmpfs with `size=75%` (12 GiB on the Mac), so this change keeps main's recipe; `docs/fedora-iso.md`'s tmpfs paragraph adds Bazaar's 1.9 GiB to the peak |
| 2 | Should fix | If kiwi's image root moved, the manifest loop would write junk lines and the build would still pass, so nothing checks that the manifest lists Bazaar | `fedora/justfile:100-101` | fixed | The recipe ends with `grep -q "^flatpak|app/io.github.kolunmi.Bazaar/"` on the manifest, which fails the build when Bazaar's line is missing |

Tests after the fixes: `just lefthook all` passes (15 checks); shellcheck 0.11.0 on `tests/vm/arch.sh`, `tests/vm/fedora.sh`, `tests/vm/installed.sh` and `fedora/iso/config.sh` passes; Prettier passes on the new site pages; `fedora/justfile` parses; the manifest loop, run on a fake image root, prints `flatpak|app/org.example.App/x86_64/stable|abc123` and `flatpak|runtime/org.gnome.Platform/x86_64/51|def456`. Image builds and VM tests are left for the `images` workflow (the PC's runner was busy).

## Round 2
Reviewed: the whole change again, with both fixes.
Verdict: Looks good. Ship. (Checked: `size=75%` and the BRE `grep -q`, where `|` is a literal, are valid inside the recipe's single-quoted `bash -c`; the new tmpfs peaks, about 7.2 GiB on x86_64 and 8.5 GiB on aarch64, fit the 75% cap on the PC and on the Mac's 12 GiB; nothing parses the manifest's `flatpak|` lines except as text: `just fedora test-install`'s awk matches only `kernel-core` and `limine`.)

## Round 3
Reviewed: the pre-commit hook's gitleaks scan of the Fedora step.
Verdict: gitleaks blocked the commit.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | gitleaks' `generic-api-key` rule flags line 8 of the committed copy of Flathub's file, its `GPGKey=` line (Flathub's public signing key), so the commit cannot pass the hook or `just gitleaks history` | `fedora/packages/stackific-desktop/flathub.flatpakrepo:8` | fixed | The copy is gone: `stackific-desktop` requires `fedora-flathub-remote` and links its file (the same key, by sha256) to `/usr/share/flatpak/remotes.d/flathub.flatpakrepo`; no allowlist |

Tests after the fixes: `just lefthook all` passes; gitleaks passes on the staged Fedora step.

## Round 4
Reviewed: the spec's link and the docs after round 3.
Verdict: Looks good. Ship. (Checked: the link target is in `fedora-flathub-remote`, which the spec requires, so it always resolves; the remote's name comes from the link's name, `flathub`, which the tests expect; the file's `Filter=` names `/usr/share/flatpak/fedora-flathub.filter` from the same package, which holds only `# Unfiltered`.)
