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
| Workflows   | GitHub Actions workflows, with actionlint                                                                                                    | `just actionlint lint`                                                               |
| Dev records | Task files, review records and prompt-trail entries follow their templates and point at each other                                           | `just dev check`                                                                     |
| Site        | Formatting, lint, types, known-vulnerable dependencies, prompt-trail screenshots, a full build                                               | `just site format-check`, `lint`, `typecheck`, `audit`, `screenshots-check`, `build` |

## On each commit message

Every commit carries a DCO `Signed-off-by:` trailer (`git commit -s`): `just lefthook dco <file>`.

## By hand

- `just lefthook all` runs every check before a commit on every tracked file, as the check workflow does.
- `just gitleaks history` scans the whole history for secrets.
- `just lefthook hooks` rewrites the git hooks after `lefthook.yml` changes.
