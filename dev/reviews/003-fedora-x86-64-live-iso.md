# 003: Fedora 45 x86_64 console live ISO with a QEMU boot test: review

Task: dev/tasks/003-fedora-x86-64-live-iso.md
Status: commit ready

## Round 1
Reviewed: the task's uncommitted change (fedora/, tests/vm/, root justfile, arch/justfile, images.yml, docs, AGENTS.md, the task file's weak-dependency update and the prompt-trail entry), with the built manifest.
Verdict: fix 1 first.

| # | Group | Finding | Where | Status | Resolution |
| --- | --- | --- | --- | --- | --- |
| 1 | Must fix | The task lists `rpm`, `rpm-libs`, `rpm-plugin-selinux`, `rpm-sequoia`, `lua-libs`, `libarchive` and `lz4-libs` as weak dependencies left out, but the manifest has all seven: `selinux-policy` requires `rpm-plugin-selinux`, which pulls the rest. The record of what is in the image was wrong. | `dev/tasks/003-fedora-x86-64-live-iso.md` (Context, weak dependencies) | fixed | Those seven move to a new "In the image anyway, as hard dependencies" line; only `rpm-plugin-audit` stays in the left-out list. The image is unchanged. |

Tests after the fixes: `just dev check` passes. The manifest check of every left-out package finds none of them (the seven hard dependencies excepted).

## Round 2
Reviewed: the whole change again after round 1's fix, with kiwi's `repository/dnf5.py` (the `sourcetype` deviation) and both boot tests.
Verdict: Looks good. Ship.
