# 009: Arch Linux ARM aarch64 live ISO boots to MangoWM with Noctalia, with KDE Plasma: review

Task: dev/tasks/009-arch-aarch64-desktop-live-iso.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change: `arch/iso/packages.aarch64`, `tests/vm/arch.sh` (renamed from `tests/vm/arch-x86_64.sh`), the removed `tests/vm/arch-aarch64.sh`, the `test` recipe in `arch/justfile`, the `boot.sh` line in `AGENTS.md`, `docs/arch-iso.md` and the prompt-trail entry. It grepped every reference to the old check-file names: only earlier task files and prompt-trail entries name them, as history.
Verdict: fix 1 first, or record it as a follow-up.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Should fix | The two package lists must match line for line, apart from the kernel, microcode and keyring lines, but only a hand-run `diff` checks it. A package added to `packages.x86_64` alone (task 006's artwork package, say) would silently miss the aarch64 image. | `arch/iso/packages.aarch64` | rejected | The fix is a pre-commit check: a recipe, a `lefthook.yml` entry and a `docs/checks.md` row. Those files are outside the Changes list, which the task's criterion "touch files outside the Changes list" forbids, and AGENTS.md makes CI checks chore work ("Repo tooling (setup, hooks, site scaffolding, CI checks) is chore work, done directly"). Left as a follow-up chore for the operator. |

Tests after the round (no code changed): `just arch iso` and `just arch test` in the runner VM, as recorded in the prompt-trail entry's Result.

## Round 2
Reviewed: the same scope again, unchanged since round 1.
Verdict: Ship. Nothing new; finding 1 stays rejected (round 1) and so counts as resolved.

## Round 3
Reviewed: the two desktop defaults the operator added to the task after round 2. One turns Noctalia's first-run welcome off: `desktop/noctalia/00-stackific.toml` installed into `/etc/skel/.config/noctalia/`. The other puts the Stackific logo on the greeter: the asset tree `/usr/share/stackific/noctalia-greeter/` in `stackific-desktop` (pkgrel 2, now depending on `noctalia-greeter`) and `NOCTALIA_GREETER_ASSETS_DIR` in greetd's `[default_session] command`. Also the two new checks in `tests/vm/arch.sh` and the docs, AGENTS.md and entry text for both. `boot.sh` has no `assets` variable for the check file's `assets` to shadow.
Verdict: Looks good. Ship.
Not checked: if a later noctalia-greeter loads more files from its assets tree than the fonts and `noctalia.svg`, ours lacks them. The boot test catches missing fonts or logo, not files a future release adds.

Tests after the round, in the runner VM: `just arch iso` exits 0 (1290 MiB, `stackific-desktop 0.1.0-2`). `just arch test` prints PASS, including `setup_wizard_enabled = false` in live's merged Noctalia config, the complete asset tree and `NOCTALIA_GREETER_ASSETS_DIR` in the running greeter's environment. The screenshots show no welcome dialog, and the Stackific logo at the bottom of the greeter.

## Round 4
Reviewed: the task rebuilt on `main` after task 006 (`74afde7`). 006 merged with a Noctalia default that turns the welcome off and with our logo on the greeter, for every Arch image. So round 3's welcome file, greeter asset tree, greetd change and their two checks were dropped as duplicates. What remains:
- `packages.aarch64` mirroring `packages.x86_64`, now with `stackific-artwork`;
- `tests/vm/arch-x86_64.sh` renamed to `tests/vm/arch.sh`, unchanged except its header;
- `tests/vm/arch-aarch64.sh` removed;
- the `test` recipe, the naming line in AGENTS.md and the docs.

The removed aarch64 file's identity, wallpaper and fastfetch checks are all in the shared file; only its console checks are gone, and they no longer apply.
Verdict: Looks good. Ship.

Tests after the round, in the runner VM: `just arch iso` exits 0 (1332 MiB, 658 packages, with `stackific-desktop 0.2.0-1`, `stackific-release 0.1.0-2` and `stackific-artwork 0.1.0-1`). `just arch test` prints PASS with every check of `tests/vm/arch.sh`, 006's greeter, wallpaper and fastfetch checks included. The screenshots show the Vesuvius wallpaper with no welcome dialog, and the greeter in our palette with the Stackific logo.

## Round 5
Reviewed: the update to `main` after task 007 (`6638f9e`). 007's fastfetch fix lists fastfetch's default modules in `branding/fastfetch.jsonc.in` (`stackific-artwork` pkgrel 2). It also added `fastfetch --pipe true | grep -c 'OS: Stackific OS'` to both `tests/vm/arch-x86_64.sh` and `tests/vm/arch-aarch64.sh`. The rename carries the x86_64 file, that check included, into `tests/vm/arch.sh`, so removing the aarch64 file loses no check. The AGENTS.md naming line merged cleanly with 007's AGENTS.md changes, which touch other lines.
Verdict: Looks good. Ship.

Tests after the round, in the runner VM: `just arch iso` exits 0 (1332 MiB, 658 packages, `stackific-artwork 0.1.0-2`). `just arch test` prints PASS, including fastfetch's logo in the accent and its `OS: Stackific OS` line.

## Round 6
Reviewed: the update to `main` after task 008 (`531db03`). 008 changed no Arch build or test input (`git diff 6638f9e 531db03 -- arch desktop branding tests/vm/boot.sh tests/vm/arch-x86_64.sh tests/vm/arch-aarch64.sh docs/arch-iso.md` is empty), so the round-5 build and boot test stand for this tree. Also reviewed: the Arch docs' manual Plasma login step, which now types any character as the password, because 008 found that noctalia-greeter refuses an empty field. And the AGENTS.md naming line, which merged cleanly with 008's AGENTS.md changes on other lines.
Verdict: Looks good. Ship.

Tests after the round: `just lefthook all` and `just dev check` pass on the Mac. No rebuild: the Arch inputs are unchanged since round 5's PASS.
