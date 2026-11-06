# Checks

Every check is a `just` recipe. Git hooks run them through [lefthook](https://lefthook.dev), mapped in `lefthook.yml`, and the `check` workflow runs the same list on every pull request and every push to `main`.

## Before each commit

Each check runs only when the staged files include something it covers.

| Check       | Covers                                                                                                                                       | Recipe                                                                               |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Hygiene     | Whitespace errors, conflict markers, files over 1 MiB (the wallpapers in `branding/wallpapers/` excepted), names of local untracked material | `just lefthook hygiene <files>`                                                      |
| Secrets     | Keys, tokens and passwords in the staged changes                                                                                             | `just gitleaks staged`                                                               |
| Justfiles   | Every justfile and module still parses                                                                                                       | `just lefthook justfiles`                                                            |
| Shell       | Shell scripts and PKGBUILDs, with shellcheck                                                                                                 | `just shellcheck lint <files>`                                                       |
| Installer   | `installer/**`: rustfmt and clippy (warnings as errors), then the unit, navigation and snapshot tests                                         | `just installer lint`, `just installer test`                                         |
| Workflows   | GitHub Actions workflows, with actionlint                                                                                                    | `just actionlint lint`                                                               |
| Dev records | Task files, review records and prompt-trail entries follow their templates and point at each other                                           | `just dev check`                                                                     |
| Site        | Formatting, lint, types, known-vulnerable dependencies, prompt-trail screenshots, a full build                                               | `just site format-check`, `lint`, `typecheck`, `audit`, `screenshots-check`, `build` |

## On each commit message

Every commit carries a DCO `Signed-off-by:` trailer (`git commit -s`): `just lefthook dco <file>`.

## By hand

- `just lefthook all` runs every check before a commit on every tracked file, as the check workflow does.
- `just gitleaks history` scans the whole history for secrets.
- `just lefthook hooks` rewrites the git hooks after `lefthook.yml` changes.

## Image tests

On a Linux host with podman and KVM, and in the `images` and `release` workflows on the self-hosted runners:

- `just arch test`: boots the Arch ISO and checks the live session ([arch-iso.md](arch-iso.md)).
- `just arch test-install` (Arch x86_64): the boot menu's entries, then the installer in a VM on a 64 GiB disk: the console path, a refused erase, an erase, a BitLocker refusal and a free-space install next to a Windows-like table, each installed system checked ([installer.md](installer.md)).
