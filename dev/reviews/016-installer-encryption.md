# 016: Every install encrypted: LUKS2 under btrfs, unlocked by the login password, with a plain screen after three wrong tries: review

Task: dev/tasks/016-installer-encryption.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (the installer's encryption and password strength, the boot-failure screen in stackific-boot on both bases, the VM tests and recipes, the docs), after both bases' `test-install` passed and their passphrase-prompt screenshots were read.
Verdict: fix 1 and 2 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | The You screen reads the password with the live session's `us` layout, the boot prompt with the chosen one: on `de` the key labelled Y stores `z`, so the disk never unlocks | `installer/src/ui/you.rs` (`draw`) | fixed | With another layout chosen, the help says so and to use only characters on the same keys in both; the user page says it too; typing in the chosen layout is under Later in `docs/installer.md` |
| 2 | Should fix | The passphrase prompt sits under about 40 boot status lines, and on Fedora dracut's "Failed to start systemd-cryptsetup@… not found" follows it on the same line | `installer/src/boot.rs` (`cmdline`) | fixed | `quiet` on the command line; the rebuilt Fedora prompt still had dracut's line, which dracut adds only for a device missing from `/etc/crypttab`, so the installer also writes `/etc/crypttab` (`disk::crypttab`); both prompts now stand alone; the VM checks expect both |

Tests after the fixes: `just installer test` (66) and `just installer lint` pass; `just arch packages`, `just arch iso` (1546 MiB), `just arch test`, `just arch test-install` and `just arch screenshots` pass; `just fedora packages`, `just fedora iso` (1498 MiB), `just fedora test-install` and `just fedora test` pass; the prompt screenshots (`luks-prompt.png`) show only the passphrase line on both bases.

## Round 2
Reviewed: the whole change after round 1's fixes (the layout warning, `quiet`, `/etc/crypttab`), with both installed systems reaching `running` (no failed unit from the crypttab entry) and the new screenshots read.
Verdict: Looks good. Ship.
