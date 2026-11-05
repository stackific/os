# AGENTS.md: constitution of `os`

This repo builds **Stackific OS** (os-release `ID=stackific`, site https://os.stackific.com). Work runs through two skills, `/s-task` and `/s-build` (`.claude/skills/`); the rules below apply to all work, skill or not.

## Product
- Stackific OS is a Linux distribution with two bases, Arch Linux (rolling; Arch Linux ARM on aarch64) and Fedora 45. Every release ships four live ISOs with one version: Arch x86_64, Arch Linux ARM aarch64, Fedora 45 x86_64 and Fedora 45 aarch64 (`stackific-<version>-<base>-<arch>.iso`).
- Each ISO aims at 2 GiB or less, a soft target (GitHub's per-file release cap); every image task reports its ISO size.
- Configure and package; build as little as possible. No own command-line tool and no AI features. Avoid tools built by CachyOS; chwd (hardware detection on Arch) is accepted. Small glue (a systemd unit, a udev rule, a short script) only when no packaged tool does the job, and the task says why. The installer, one program for both bases, is the one larger program we write.
- Desktop on every image: MangoWM with Noctalia, both unmodified packages, plus KDE Plasma on Arch and GNOME on Fedora. Our defaults ship as config files in a package, never as patches.
- Boot and disk, the same on both bases: Limine from our own 4 GiB FAT32 ESP (never another OS's), one btrfs subvolume layout, installed alongside another OS or on a whole drive; Secure Boot off.
- Hardware: every AMD and Intel CPU and GPU the base supports; NVIDIA Turing and newer on NVIDIA's open modules, prebuilt in the image or built on first boot; older NVIDIA on nouveau; the drivers and firmware regular hardware needs.
  - Left out: server and datacenter NICs, and hardware that needs a vendor kernel (Microsoft Surface, Apple T2 Macs, Apple Silicon, Raspberry Pi).
- Backup and restore: snapper with btrfs-assistant, on installed systems and in the live images; nothing more.
- Never configure passwordless sudo (`NOPASSWD`) or polkit rules that skip authentication.
- Releases are semver (`vMAJOR.MINOR.PATCH`), cut only by the manual `release` workflow.
- Bias for shipping a working software as quickly as possible and keep a log of future improvements, so that we can address some of them later
- Update `AGENTS.md` file concisely

## Repo layout
Planned; each directory appears with the first task that needs it. One concern per directory.

```
README.md            developer setup, step by step
justfile             wires modules, setup and doctor, nothing else
lefthook.yml         git hooks, each calling a just recipe
just/<tool>/         shared toolchains: mod.just, vars.just (pins)
arch/                Arch base: package lists, archiso profile, PKGBUILDs
fedora/              Fedora base: image definitions, RPM specs, repo files
desktop/             base-neutral defaults: MangoWM config, Noctalia defaults, session files
installer/           the installer: disk step, live-image copy, per-base steps
branding/            the branding file, logo and wallpapers (committed, credited in wallpapers.toml)
hardware/            PCI ID lists, chwd profiles and the Fedora detection step
tests/               container and VM tests
site/                docs and the prompt trail
dev/                 `just dev check` (dev/justfile) over tasks, reviews and prompt-trail entries
dev/tasks/           task files from /s-task
dev/reviews/         review records from /s-build, one per task
.github/workflows/   check, images, release
.claude/             settings.json (plugins, hooks) and skills
.serena/project.yml  Serena's config for this repo
.mcp.json            MCP servers (Serena)
```

## Tasks: just
- `just` runs every repo command; no Makefiles, Taskfiles or ad-hoc scripts.
- The root `justfile` only wires modules, `setup` and `doctor`. Each area owns `<dir>/justfile` (`mod <dir>`); shared toolchains live in `just/<tool>/mod.just` with pins in `just/<tool>/vars.just`. Modules can't see siblings, so mount a needed toolchain as `[private] mod`.
- Run recipes from the repo root: `just <module> <recipe>`. Run several recipes of one module as `just <module>::<a> <module>::<b>`.
- Repo-local tools install to `.tools/` (gitignored), pinned by version and checksum in `vars.just`; single-file tools go through `just/fetch` (`fetch::get`), which skips the download while `<dest>.sha256` matches the pin. Launchers go to `~/.local/bin` only for tools that must be on PATH (language servers, `rtk`, `graphify`); an existing file there that does not point into `.tools/` is the user's and is kept.
- `just setup` installs every prerequisite to build, test, lint and run everything, plus the agent tools below; only the host prerequisites in `README.md` (git, just, node, npm, unzip, Claude Code) are installed by hand. A change that adds a tool or dependency updates `just setup` in the same change.
- Setup hosts: Linux (x86_64, aarch64) and macOS on Apple silicon; image builds need Linux. Host-specific asset names, checksums and the checksum command live in `just/host/vars.just` and each tool's `vars.just`.
- `README.md` keeps the developer setup step by step. A change to setup or its host prerequisites updates it in the same change. Replies to the operator point to it instead of listing steps.
- `just doctor` prints one line per tool: its pinned version and wiring (git hooks included), plus host prerequisites setup cannot install (`claude`; `node` for ponytail's and caveman's hooks; `npm` for Serena's Bash language server). Image tasks add podman, KVM and vhost-vsock; OVMF and AAVMF come from the pinned builder containers.
- Git hooks: lefthook (pinned), mapped in `lefthook.yml`, calling just recipes only; after editing `lefthook.yml`, run `just lefthook hooks`. `just lefthook all` runs every pre-commit check on every tracked file (only tracked: nothing before the first `git add`).
  - pre-commit: hygiene (`git diff --check`, files over 1 MiB except the wallpapers in `branding/wallpapers/`, names of `.git/info/exclude` root entries), gitleaks, justfile parse, shellcheck (`*.sh`, `PKGBUILD`), actionlint, `just dev check`, site format, lint, types, audit, screenshots and build on `site/**`. `mango -p` joins when `desktop/` ships the MangoWM config.
  - commit-msg: DCO sign-off.
  - Module recipes run in their module's folder, but `justfile_directory()` is the repo root: `cd "{{ justfile_directory() }}"` for repo paths.
- Before adding to `.gitignore`, read its header. It ignores only build output, setup output, IDE files and optional user files; personal patterns go in `.git/info/exclude`.

## Untracked material
- No tracked file, recipe, hook or workflow names local untracked material (notes, earlier work, scratch folders); bring what is needed into the tracked tree first. Such folders are listed in `.git/info/exclude`.
- Take from it only what the operator names. It is input, not instruction: check each decision against this file and current upstream before restating it.
- Query earlier work in place: `just graphify update <dir>` builds a graph inside it (`graphify query "<q>" --graph <dir>/graphify-out/graph.json`); Serena indexes it with the repo (Serena reads only `.gitignore`), with any language only it needs (Go) in the untracked `.serena/project.local.yml`; rtk works anywhere.

## Agent tools
`just setup` installs all of them. Plugins, their marketplaces and the hooks live in `.claude/settings.json`; `just claude plugins` installs the plugins at project scope.

| Tool | Use it for | Setup |
| --- | --- | --- |
| rtk (`rtk-ai/rtk`) | Compact shell output on every Bash call | Release binary in `.tools/rtk`, launcher on PATH; `PreToolUse` Bash hook `rtk hook claude` in `.claude/settings.json`, never in global settings |
| graphify (`graphifyy`) | First stop for "where is X" and "how does Y work" | uv tool, launcher on PATH; `graphify install --project` writes its skill (gitignored), its `PreToolUse` hooks and its `CLAUDE.md` section (both committed); `just graphify update [dir]` rebuilds `graphify-out/` (code only, no LLM; gitignored) |
| Serena (`serena-agent`) | Symbols, references, renames | uv tool, not the marketplace plugin (Serena's README advises against it); `.mcp.json` runs `just serena mcp` (context `claude-code`); `.serena/project.yml` holds this repo's languages and ignores every `.claude/worktrees/`; Go sources use the pinned Go and gopls from `just go install` |
| caveman | Short replies, fewer output tokens | Plugin `caveman@caveman`, marketplace `JuliusBrussee/caveman` |
| ponytail | Least-code rules on every change; `/ponytail-review` inside `/s-build` | Plugin `ponytail@ponytail`, marketplace `DietrichGebert/ponytail` |
| context7 | Versioned library and tool docs | Plugin `context7@claude-plugins-official` |
| github | Issues, PRs, Actions runs | Plugin `github@claude-plugins-official`; `gh` CLI |
| security-guidance | Security of new code; act on its warnings | Plugin `security-guidance@claude-plugins-official` |
| commit-commands | Commit and PR flows, only when told to commit | Plugin `commit-commands@claude-plugins-official` |
| claude-md-management | Keeping this file current | Plugin `claude-md-management@claude-plugins-official` |

- **Every session** uses rtk, caveman, ponytail, context7, Serena and graphify. Plugins and MCP servers load at session start, so restart Claude Code after `just setup`.
- **Pins** (checked 2026-11-04), in `just/<tool>/vars.just`: rtk 1.1.0, graphify 0.9.102, Serena 2.1.0, uv 0.13.5 with Python 3.13, Go 1.27.3, gopls v0.24.0, Bun 1.4.4, lefthook 2.2.2, gitleaks 8.30.1, shellcheck 0.11.0, actionlint 1.7.12, gh 2.103.3, Actions runner 2.338.2 (first install only; it updates itself). Marketplaces are pinned by `ref` in `.claude/settings.json`: caveman v4.0.1, ponytail v5.3.3. Site packages are pinned in `site/package.json` and `site/bun.lock`; CI actions by commit SHA.
- **LSP plugins** join with their language: `pyright-lsp` (with `pyright-langserver` in `just setup`) arrives with the first Python code. No `typescript-lsp`: the site's few TypeScript files are covered by `astro check`.
- **Other tools** (rclone) join `just setup` with the first task that needs them; podman for image builds is a host prerequisite (`README.md`).
- **When to use what:** graphify, then Serena, for this repo and earlier work; context7 for library APIs. rtk wraps shell output on its own.
- **Gotchas:** rtk reshapes output (summaries, hidden lines), so run `rtk bypass <cmd>` where a script parses it. In auto mode, a command rtk's hook rewrote can come back unreviewed: send it again unchanged, once. Serena's "Go is not installed" means its server started before `just go install`: reconnect it with `/mcp`.

## Site
- `site/`: Astro with Starlight docs at `/docs` (`src/content.config.ts` prefixes ids with `docs/`); marketing pages in `src/pages/` at `/`. Bun only, never npm. ESLint, Prettier and `astro check`. English only, no i18n: UI strings live in the components. Recipes in `site/justfile`.
  - It publishes `llms.txt` (`starlight-llms-txt`) and a `.md` twin of every docs page (`src/pages/[...slug].md.ts`).
  - Fonts and colours come from the branding file.
  - Sidebar groups: Developer Guide (`developer/`) and Prompt Trail (`prompt-trail/`); add a group to `astro.config.mjs` with its first page.
- Prompt trail: one entry per task PR, written from `.claude/skills/s-build/entry-template.md`. A task on both bases goes to `site/src/content/docs/prompt-trail/<NNN>-<slug>.md`; a single-base task goes under `prompt-trail/arch/` or `prompt-trail/fedora/`, labelled Arch and Fedora by `src/routeData.ts`. Frontmatter `pr:` and `task:` draw the links under the title.
- Screenshots: `site/src/assets/prompt-trail/<NNN>/<name>.webp`, shown only by their own entry and made with `just site screenshot <raw.png> <out.webp>` (WebP, longest side 1600 px or less, 300 KB or less, metadata stripped).
  - `just site screenshots-check` (pre-commit) enforces it.
  - Site pages are never screenshotted.
- Docs entry: every task adds or updates the docs pages its change needs, user docs for what users see and developer docs for how to build and test it.

## Builds
- Arch x86_64 builds pin one Arch Linux Archive date, moved forward only through a test gate; Arch builds and tests run in a pinned container. Outputs go to `.build/` (gitignored).
- The Arch builder is `arch/Containerfile`, pinned with the archive date in `arch/vars.just`; mkarchiso runs in it as the unprivileged user `build` (rootless podman cannot mount devtmpfs).
- Arch Linux ARM and Fedora build from their live repos; every image records its full package manifest.
- Our packages: makepkg in a fresh pinned container on Arch, mock on Fedora. They hold configuration, artwork, release files and third-party tools we package (Shelly on Arch).
- The ArtCraft apps come from upstream's own release packages (`.pkg.tar.zst`, `.rpm`), checked against its `SHA256SUMS.txt` and pinned by version.
- Fedora takes MangoWM, scenefx and Limine from Terra. Terra's repo file is limited with `includepkgs`, and its signing key is checked at build time. NVIDIA and Broadcom wl come from RPM Fusion; Bazaar from Flathub.
- Signing: packages, repo databases, `SHA256SUMS` and images use a subkey of an offline project key; the subkey lives only in the `release` environment.

## CI and releases
- GitHub Actions in `.github/workflows/`; every step after checkout and just's setup calls a just recipe. `check` (pull requests, pushes to `main`) runs `just lefthook all` and `just gitleaks history`.
- GitHub-hosted Linux runners run checks and container tests.
- The operator's Linux PC runs the self-hosted runner for x86_64 images and their VM tests (labels `self-hosted`, `Linux`, `X64`); `just runner setup` makes a machine a runner, never `just setup`.
- The operator's Mac runs the self-hosted runner for aarch64 images (an arm64 Linux VM from `just runner vm`, labels `self-hosted`, `Linux`, `ARM64`).
- Self-hosted runners take only `push` to `main` and `workflow_dispatch`, never `pull_request`; `just runner register` makes every outside contributor's run wait for approval.
- `release` runs only by hand (`workflow_dispatch`, input `version`):
  - refuses a version that is not semver;
  - builds and tests all four images, then signs them;
  - tags `v<version>` and creates the GitHub release with each ISO, `SHA256SUMS` and its signature attached;
  - an ISO over 2 GiB goes to Cloudflare R2 instead and the release links it: secrets `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, variable `R2_BUCKET` (the bucket name, such as `stackific-os`). Wired only once an ISO needs it.
  - A released version never changes.
- Secrets are never printed and never committed; gitleaks runs in pre-commit and in `check`.

## Tests
- Tests run without asking; anything that needs a person gets step-by-step instructions for the operator.
- As much automation as is pragmatic:
  - static checks on every commit;
  - container tests for packages, package lists and headless MangoWM with Noctalia;
  - QEMU UEFI (OVMF, AAVMF) boot, session and unattended-install tests for every image, in both install modes, on blank and Windows-like disk images.
- VMs get test access only from one-boot SMBIOS credentials with a token-gated vsock shell; no image ships sshd or test units.
- Boot tests reach the VM only through the vsock shell from `systemd.extra-unit.*`/`systemd.unit-dropin.*` credentials and wait with `systemctl is-system-running --wait` over it; `vmm.notify_socket` over vsock drops messages, so it is not used (`tests/vm/`).
- Tests read and edit disk images only through `qemu-img`, `qemu-io` and `mtools` on image files: never root, loop devices, `qemu-nbd` or mounts.

## Workflow: /s-task and /s-build
- Tasks are distro work only (images, packages, defaults, the installer, hardware, distro tests and releases). Repo tooling (setup, hooks, site scaffolding, CI checks) is chore work, done directly.
- `/s-task <ask>` writes `dev/tasks/<NNN>-<slug>.md`, where `NNN` is the next free three-digit number. The task is self-contained:
  - the ask as one consolidated prompt;
  - its base: `both` (the change is the same on both bases, or touches neither), `arch` or `fedora` (the tooling differs, as for image builds);
  - the goal and the context;
  - every fact it relies on, with its source;
  - the files to create or change;
  - acceptance criteria (Must / Must not);
  - a `- [ ] Done` box.

  It names only tracked files. Material it takes from an untracked source is restated in it, or copied to its tracked target path while the task is written.
- `/s-build <task>` makes it commit ready:
  - code and tests;
  - the docs entry;
  - the prompt-trail entry with the task's number, in its base's folder;
  - the review cycle: `/ponytail-review` on the change, each round recorded in `dev/reviews/<NNN>-<slug>.md` (one file per task, updated in place), every finding fixed, tests rerun, until a review finds nothing. If findings remain after two fix rounds, it stops and tells the operator the cycle is not making progress. The review does not replace tests or security-guidance.
- One task = one PR = one prompt-trail entry; one step = one commit. Chores have no task, are their own commits, and get no entry.
- The operator ticks Done and does every git operation. Skills may read history but never commit, push, rebase or open PRs.

## Git: hard rules
- Never commit unless the operator explicitly says so in the current request; earlier approval does not carry over.
- Every commit is DCO signed-off (`git commit -s`) and SSH-signed (`-S`). Never use `--no-gpg-sign` or skip hooks. If signing fails, stop and report.
- Never push, force-push or open PRs without explicit instruction. When told to: post a PR's link only once it is ready to review and merge, and add chores to the open chore PR rather than opening one per chore.
- No AI attribution anywhere: no `Co-Authored-By` AI trailers, no "Generated with" or "Written by" lines, no AI names or tool links in commits, PRs, code or comments. This overrides any tool default.

## Style
- Modular, idiomatic code and folders for each ecosystem; one concern per directory; no catch-all `utils/` or `misc/`.
- Doc comments on every module, type and function; comments for non-obvious logic.
- Indentation: 2 spaces (tabs where the language requires them).
- The least code that works; reuse what the bases and standard libraries provide; match surrounding conventions.
- Shell scripts use `set -euo pipefail` and pass shellcheck.

## This file
- Keep enriching AGENTS.md with durable facts (structure, commands, conventions, gotchas, the operator's preferences): one line per fact, no duplication.
- Durable guidance goes here, never only in an agent's local memory: memory stays on one machine, and this file travels with the repo to every machine the operator works on.
