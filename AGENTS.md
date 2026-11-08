# AGENTS.md: constitution of `os`

This repo builds **Stackific OS** (os-release `ID=stackific`, site https://os.stackific.com). Work runs through two skills, `/s-task` and `/s-build` (`.claude/skills/`); the rules below apply to all work, skill or not.

**First, every session:** check that each MCP server in `.mcp.json` and each plugin, MCP server and hook in `.claude/settings.json` loaded. Name any that did not, with its error, loudly in your first reply, before other work.

## Product
- Positioning (the operator's words): an "Open Prompt", the most vibe-codable OS foundation for making your own Linux. Every change starts as a published prompt (the prompt trail), so anyone can fork the repo, change a prompt and rebuild their own Linux.
- Stackific OS is a Linux distribution with two bases, Arch Linux (rolling; Arch Linux ARM on aarch64) and Fedora 45. Every release ships four live ISOs with one version: Arch x86_64, Arch Linux ARM aarch64, Fedora 45 x86_64 and Fedora 45 aarch64 (`stackific-<version>-<base>-<arch>.iso`).
- The four images are one product: the same branding, messaging, defaults, files at the same paths, live session and boot-test checks. They differ only where a base or architecture forces it (the second desktop, package names, kernel, firmware, build tooling), and the task names each difference. A change to one image lands on all four, or the task says which image task carries it.
- Each ISO aims at 2 GiB or less, a soft target (GitHub's per-file release cap); every image task reports its ISO size.
- Configure and package; build as little as possible. No own command-line tool and no AI features. Avoid tools built by CachyOS; chwd (hardware detection on Arch) is accepted. Small glue (a systemd unit, a udev rule, a short script) only when no packaged tool does the job, and the task says why. The installer, one program for both bases, is the one larger program we write.
- Desktop on every image: MangoWM with Noctalia, both unmodified packages, plus KDE Plasma on Arch and GNOME on Fedora. Our defaults ship as config files in a package, never as patches.
  - MangoWM starts Noctalia through `stackific-noctalia.service` (package `stackific-desktop`), wanted by `mango-session.target`, which MangoWM starts itself; MangoWM's config stays stock. Each user's `~/.config/mango/config.conf` (the live image's for `live`, the installer's for installed users; never a package) sources it and adds `Super+Space` for Noctalia's launcher (`noctalia msg panel-toggle launcher`, operator 2026-11-06).
  - `stackific-desktop` exports `WLR_RENDERER_ALLOW_SOFTWARE=1` (`/etc/profile.d/`), so MangoWM starts on software rendering too (VMs without 3D, GPUs without a working driver).
  - Live images autologin the user `live` (empty password, group `wheel`) into MangoWM through greetd's `initial_session`; noctalia-greeter is the login screen. It refuses an empty password field, so `live` logs in there with any typed character (operator choice over giving `live` a password; `site/src/content/docs/user/installation.md` says so).
  - noctalia-greeter shows our look, not Noctalia's: `stackific-desktop` ships `/usr/share/stackific/greeter.toml` (our palette, Vesuvius), linked from `/var/lib/noctalia-greeter/greeter.toml` by a tmpfiles `L` line, so an administrator's own file wins. Our logo replaces Noctalia's through the greeter's `NOCTALIA_GREETER_ASSETS_DIR` (`/etc/profile.d/stackific-greeter.sh`): `/usr/share/stackific/greeter-assets/` links `noctalia.svg` to our icon and `fonts` to the greeter's own.
  - Plasma (Arch's second desktop) shows our wallpaper through our look-and-feel package `org.stackific.desktop` (Breeze with `[Wallpaper] Image` set to our painting, packaged under `/usr/share/wallpapers/stackific/`), which `/etc/xdg/kdeglobals` selects; all in `stackific-desktop` on Arch. GNOME (Fedora's) gets it from `stackific-desktop`'s `20_stackific.gschema.override` (background, dark background, lock screen), which also puts the installer first in GNOME's dash (`favorite-apps`; GNOME has no desktop icons).
  - Noctalia reads config only from `~/.config/noctalia/` (no system layer), so `stackific-desktop` ships our defaults as `/usr/share/stackific/noctalia.toml` (setup wizard off, wallpaper folder `/usr/share/backgrounds/stackific`, default Dahl's "An Eruption of Vesuvius"), included from each user's `~/.config/noctalia/00-stackific.toml`, which comes from `/etc/skel`.
- `stackific-release` owns the os-release (`ID=stackific`, `ID_LIKE` and `BUILD_ID` or `VERSION_ID` set per base, `LOGO=stackific`, the icon being `branding/logo.svg` filled with the accent at build time). On Arch it owns `/etc/os-release`, which takes precedence over the base's `/usr/lib/os-release`, so base packages and `/etc/arch-release` stay as shipped. On Fedora it owns `/usr/lib/os-release` in place of `generic-release`, and `generic-release-common`'s `/etc/os-release` symlink points at it.
- Base labels, the only words for what an image is built from (Arch's and Fedora's trademark rules): `Based on Arch Linux` (Arch x86_64), `Based on Arch Linux ARM` (Arch aarch64), `Fedora Remix` (both Fedora images); Fedora's name never follows "based on" (that needs the Fedora Council's permission), and no base logos. Fedora's notice (not provided or supported by the Fedora Project; official Fedora at https://fedoraproject.org/) and Red Hat's trademark line go only where the text uses a Fedora mark, such as "Fedora Remix" (the site's footer carries the line on every docs page); Fedora's guidelines tie the notice to the Remix marks and the line to pages that use a mark. File names (`stackific-<version>-fedora-x86_64.iso`) are not such a use, so text that names no mark, as the README and the release notes do, carries only "Stackific OS is not affiliated with or endorsed by Arch Linux, Arch Linux ARM, the Fedora Project or Red Hat." (operator 2026-11-06).
- `branding/branding.env` is the one branding file (shell `KEY="value"`: name, ID, URLs, accent `#007a55`); package builds render `*.in` templates (`branding/`, `desktop/`) from it with `envsubst` and an explicit key list. A branding edit means new sha256s and a `pkgrel` bump in each PKGBUILD that sources it (`Release` in each spec).
- `stackific-artwork` ships the wallpapers (`branding/wallpapers/`, credited in `branding/wallpapers.toml`) to `/usr/share/backgrounds/stackific/`, and fastfetch's logo in the accent through `/etc/xdg/fastfetch/config.jsonc`; it depends on `fastfetch`. Every image installs `stackific-release` and `stackific-artwork`, and its boot test checks fastfetch's logo.
- Boot and disk, the same on both bases: Limine from our own 4 GiB FAT32 ESP (never another OS's), one btrfs subvolume layout, installed alongside another OS or on a whole drive; Secure Boot off.
  - Boot chain (installer step 8, task 013): the kernel and initramfs reach the ESP through the base's own tools (Arch: mkinitcpio's pacman-hook script in `arch-chroot`, default preset, no fallback image, no UKI); Limine's `BOOTX64.EFI` at `\EFI\limine\` and `\EFI\BOOT\`; one `limine.conf` at the ESP's root (fixed kernel paths, `root=UUID=… rootflags=subvol=/@ rw`); the firmware entry labelled os-release's `NAME`, created first in `BootOrder` at install only, stale entries for our loader on vanished partitions deleted; `stackific-boot`'s pacman hook re-copies Limine after `limine` upgrades. No limine-entry-tool (AUR).
  - aarch64 (task 018): Limine's program is `BOOTAA64.EFI` (the installer picks it at build time, stackific-boot by `uname -m`). Arch Linux ARM: `linux-aarch64` owns `/boot/Image`, which mkarchiso empties from the live root, so the installer copies the ISO's `arch/boot/aarch64/vmlinuz-linux` to `/boot/Image` and runs `mkinitcpio -P`; `limine.conf` names `boot():/Image`. Fedora: the aarch64 kernel is EFI zboot, which Limine's `linux` protocol refuses, so the plugin writes `protocol: efi` entries with `cmdline: initrd=\initramfs-<v>.img …`.
  - Windows in Limine's menu (task 015), after the system's entries: "Windows" (`efi_boot_entry` to the firmware's "Windows Boot Manager", written only when that entry exists) and "Windows, started directly" (`efi`, `guid(<Windows ESP PARTUUID>)`); Arch appends them to `limine.conf`, Fedora's installer writes `/etc/kernel/limine-windows.conf`, which our kernel-install plugin appends every time. Windows' ESP and firmware entry are never written.
  - Fedora (task 014): kernel-install with `layout=other`, `BOOT_ROOT=/boot` (`stackific-boot`'s drop-in), Fedora's own dracut plugin for `/boot/initramfs-<version>.img`, and our plugin `90-stackific-limine.install` copying `vmlinuz-<version>` and rewriting `limine.conf` with every kernel on the ESP, newest first, command line from `/etc/kernel/cmdline` (the installer writes it); an RPM trigger re-copies Limine. The installer removes the live image's GRUB (and the installer) with `dnf5 --installroot` before kernel-install runs in a plain `chroot` (proc, sys, `/dev` and a non-recursive `/run` bind), then relabels the target with `setfiles` (SELinux stays enforcing; no `/.autorelabel`).
  - Encryption (task 016), always on: LUKS2 (cryptsetup's defaults) on the system partition, btrfs inside; the login password is the passphrase, zxcvbn score 3 or more; command line `rd.luks.uuid=<uuid> root=/dev/mapper/luks-<uuid> rootflags=subvol=/@ rw quiet`; the systemd initramfs (Arch: the installer's `/etc/mkinitcpio.conf.d/stackific.conf` adds `sd-encrypt`; Fedora: dracut's `systemd-cryptsetup`); passphrase only, no TPM; after three wrong passphrases stackific-boot's initramfs screen (`installer/initramfs/`) replaces the emergency shell and restarts on Enter.
  - Subvolumes, flat siblings of `@`, all mounted `noatime,compress=zstd:1`: `@` `/`, `@home`, `@root`, `@srv`, `@cache` `/var/cache`, `@tmp` `/var/tmp`, `@log` `/var/log`, `@swap` `/swap`, `@flatpak` `/var/lib/flatpak`, `@libvirt` `/var/lib/libvirt/images`, `@containers` `/var/lib/containers`, `@docker` `/var/lib/docker`; `@libvirt` and `@swap` NOCOW; the ESP at `/boot`; fstab by UUID.
- The installer: `installer/`, one Rust program with a ratatui text interface for both bases (package `stackific-installer`), base differences behind its `Base` trait, every command through its `Runner` trait (faked in tests and `just installer demo`). Interactive or unattended (`--config <file>`, TOML); into a drive's largest free space or onto a whole drive. It opens from the live desktop's "Install Stackific OS" launcher (foot, `run0`) and from the boot menu's Install entry (`stackific-installer.target`, tty1); Display (text size, motion, contrast) is its first screen; a fixed 80 × 24 frame, centered; its design table is in `docs/installer.md`. On all four images (x86_64: Arch since task 012, Fedora since task 014; aarch64 since task 018); Fedora's Install entry is in the ISO's GRUB menu through kiwi's `editbootconfig` script `fedora/iso/edit-boot.sh`. The copy leaves out the live root's `/boot` and `/dev` contents (kiwi's unlabelled device nodes fail rsync under SELinux).
  - Next to Windows (task 015): BitLocker on any listed drive the install keeps (`blkid -p`, protectors from `cryptsetup bitlkDump`) gets a BitLocker screen with the recovery key IDs and needs the typed `I have my key` (`[disk] bitlocker`); Windows is found read-only by `EFI/Microsoft/Boot/bootmgfw.efi` on an ESP; Intel RST RAID members are left out and BIOS-mode Windows (MBR) refuses the free space, each with guidance; no os-prober, no entries for other Linux installs. User page: `site/src/content/docs/user/windows.md`.
- Secret Service (task 017, SPEC decision 18): oo7 on both bases in every session (MangoWM, Plasma, GNOME), started by `stackific-desktop`'s `default.target.wants/oo7-daemon.service` link and unlocked at login by `pam_oo7`: Fedora's greetd PAM file has it; on Arch `stackific-desktop` ships `/etc/pam.d/stackific-greetd`, selected by `[general] service` in the image's greetd config.
- GNOME Disks is the live images' partition editor: native Wayland in MangoWM, Plasma and GNOME, runs as the user with polkit asking per change (no root GUI, no `xhost`), the same app on all four images; it cannot move partitions, so shrinking Windows is left to Windows. `ntfsprogs` gives it `ntfsresize`; `breeze-gtk` and `kde-gtk-config` give GTK apps Plasma's look; Noctalia's polkit agent is on in our defaults.
- Hardware: every AMD and Intel CPU and GPU the base supports; NVIDIA Turing and newer on NVIDIA's open modules, prebuilt in the image or built on first boot; older NVIDIA on nouveau; the drivers and firmware regular hardware needs.
  - Arch x86_64 (task 019): `linux` and `linux-lts` (Limine's second entry `/<NAME> (LTS kernel)`; the live ISO boots `linux` only: its live-only `stackific-live-lts.hook` drops `linux-lts` from `/boot`, the installer removes the hook; the live initramfs is xz), Arch's prebuilt `nvidia-open`, `nvidia-open-lts` and `nvidia-utils` on every image and install, no DKMS; no GPU module in any initramfs (no `kms` hook on x86_64: `autodetect` would add nouveau). `stackific-hardware` (`hardware/`) ships `stackific-gpu-generator`: at every boot, NVIDIA graphics all in `hardware/nvidia-legacy.ids` get a `/run/modprobe.d/nvidia-utils.conf` that blacklists NVIDIA's modules (nouveau loads) and, as the only GPU, MangoWM's low effects via `/run/stackific/mango-gpu.conf`, which every user's MangoWM config includes with `source_optional=`; any NVIDIA display makes greetd wait for udev to settle. No chwd so far (task 021 decides). `just hardware test` checks the generator; `docs/hardware.md`.
  - Left out: server and datacenter NICs, and hardware that needs a vendor kernel (Microsoft Surface, Apple T2 Macs, Apple Silicon, Raspberry Pi).
- Backup and restore: snapper with btrfs-assistant, on installed systems and in the live images; nothing more.
- Never configure passwordless sudo (`NOPASSWD`) or polkit rules that skip authentication.
- Releases are semver (`vMAJOR.MINOR.PATCH`), cut only by the manual `release` workflow.
- What users read (README, release notes, site user docs) names the desktop MangoWM; Noctalia appears there only in paths and settings; the prompt trail is exempt.
- Bias for shipping a working software as quickly as possible and keep a log of future improvements, so that we can address some of them later
- Update `AGENTS.md` file concisely

## Repo layout
Planned; each directory appears with the first task that needs it. One concern per directory.

```
README.md            front page: what Stackific OS is, the four images, links
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
release/             GitHub Releases for the images (recipes the release and images workflows call)
site/                user docs and the prompt trail
docs/                developer notes: setup (setup.md), how to build, test and check (Markdown, not published)
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
- Repo-local tools install to `.tools/` (gitignored), pinned by version and checksum in `vars.just`; single-file tools go through `just/fetch` (`fetch::get`), which skips the download while `<dest>.sha256` matches the pin. Launchers go to `~/.local/bin` only for tools that must be on PATH (language servers, `rtk`, `graphify`, `serena`); an existing file there that does not point into `.tools/` is the user's and is kept. When `~/.local/bin` is not on PATH, the launcher step adds it to the shell profiles (`uv tool update-shell`); recipes that look up launchers prepend it for the current shell.
- `just setup` installs every prerequisite to build, test, lint and run everything, plus the agent tools below; only the host prerequisites in `docs/setup.md` (git, just, node, npm, unzip, Claude Code; Lima on macOS) are installed by hand. On macOS it also makes or starts the Fedora 45 build VM (`just runner vm`), which it never registers as a runner. A change that adds a tool or dependency updates `just setup` in the same change.
- Setup hosts: Linux (x86_64, aarch64) and macOS on Apple silicon; image builds need Linux. Host-specific asset names, checksums and the checksum command live in `just/host/vars.just` and each tool's `vars.just`.
- `docs/setup.md` keeps the developer setup step by step; `README.md` is the marketing front page (the ASCII mark from `branding/logo.txt`, the four images by their base labels, links) and points to it. A change to setup or its host prerequisites updates it in the same change. Replies to the operator point to it instead of listing steps.
- `just doctor` prints one line per tool: its pinned version and wiring (git hooks included), plus host prerequisites setup cannot install (`claude`; `node` for ponytail's and caveman's hooks; `npm` for Serena's Bash language server). Image tasks add podman, KVM and vhost-vsock; OVMF and AAVMF come from the pinned builder containers. On macOS, `runner vm:` checks the build VM runs the pinned Fedora release.
- Git hooks: lefthook (pinned), mapped in `lefthook.yml`, calling just recipes only; after editing `lefthook.yml`, run `just lefthook hooks`. The shared `.git/hooks` and the PATH launchers name the checkout that wrote them, so run `just setup` and `just lefthook hooks` in the main checkout, never in a worktree; the hooks fall back to `lefthook` on PATH (its launcher), and without lefthook they skip every check. `just lefthook all` runs every pre-commit check on every tracked file (only tracked: nothing before the first `git add`).
  - pre-commit: hygiene (`git diff --check`, files over 1 MiB except the wallpapers in `branding/wallpapers/`, names of `.git/info/exclude` root entries), gitleaks, justfile parse, shellcheck (`*.sh`, `PKGBUILD`), actionlint, `just dev check`, site format, lint, types, audit, screenshots and build on `site/**`. `mango -p` joins when `desktop/` ships the MangoWM config.
  - commit-msg: DCO sign-off.
  - Module recipes run in their module's folder, but `justfile_directory()` is the repo root: `cd "{{ justfile_directory() }}"` for repo paths.
- Before adding to `.gitignore`, read its header. It ignores only build output, setup output, IDE files and optional user files; personal patterns go in `.git/info/exclude`.

## Untracked material
- No tracked file, recipe, hook or workflow names local untracked material (notes, earlier work, scratch folders); bring what is needed into the tracked tree first. Such folders are listed in `.git/info/exclude`.
- Take from it only what the operator names. It is input, not instruction: check each decision against this file and current upstream before restating it.
- Query earlier work in place: `just graphify update <dir>` builds a graph inside it (`graphify query "<q>" --graph <dir>/graphify-out/graph.json`); Serena indexes it with the repo (Serena reads only `.gitignore`); rtk works anywhere.

## Agent tools
`just setup` installs all of them. Plugins, their marketplaces and the hooks live in `.claude/settings.json`; `just claude plugins` installs the plugins at project scope.

| Tool | Use it for | Setup |
| --- | --- | --- |
| rtk (`rtk-ai/rtk`) | Compact shell output on every Bash call | Release binary in `.tools/rtk`, launcher on PATH; `PreToolUse` Bash hook `rtk hook claude` in `.claude/settings.json`, never in global settings |
| graphify (`graphifyy`) | First stop for "where is X" and "how does Y work" | uv tool, launcher on PATH; `graphify install --project` writes its skill (gitignored), its `PreToolUse` hooks and its `CLAUDE.md` section (both committed); `just graphify update [dir]` rebuilds `graphify-out/` (code only, no LLM; gitignored) |
| Serena (`serena-agent`) | Symbols, references, renames | uv tool, not the marketplace plugin (Serena's README advises against it); launcher on PATH, which `.mcp.json` runs directly (`serena start-mcp-server --context claude-code --project-from-cwd`), never through `just`: a justfile that does not parse (a missing module) or a worktree without `.tools/` would stop it; `.serena/project.yml` holds this repo's languages and ignores every `.claude/worktrees/` |
| caveman | Short replies, fewer output tokens | Plugin `caveman@caveman`, marketplace `JuliusBrussee/caveman` |
| ponytail | Least-code rules on every change; `/ponytail-review` inside `/s-build` | Plugin `ponytail@ponytail`, marketplace `DietrichGebert/ponytail` |
| context7 | Versioned library and tool docs | Plugin `context7@claude-plugins-official` |
| github | Issues, PRs, Actions runs | Plugin `github@claude-plugins-official`; `gh` CLI |
| security-guidance | Security of new code; act on its warnings | Plugin `security-guidance@claude-plugins-official` |
| commit-commands | Commit and PR flows, only when told to commit | Plugin `commit-commands@claude-plugins-official` |
| claude-md-management | Keeping this file current | Plugin `claude-md-management@claude-plugins-official` |

- **Every session** uses rtk, caveman, ponytail, context7, Serena and graphify. Plugins and MCP servers load at session start, so restart Claude Code after `just setup`.
- **Pins** (checked 2026-11-04), in `just/<tool>/vars.just`: Rust 1.99.1 (Arch's and Fedora 45's release) through rustup 1.29.3 (checked 2026-11-06), rtk 1.1.0, graphify 0.9.102, Serena 2.1.0, uv 0.13.5 with Python 3.13, Bun 1.4.4, lefthook 2.2.2, gitleaks 8.30.1, shellcheck 0.11.0, actionlint 1.7.12, gh 2.103.3, Actions runner 2.338.2 (first install only; it updates itself). Marketplaces are pinned by `ref` in `.claude/settings.json`: caveman v4.0.1, ponytail v5.3.3. Site packages are pinned in `site/package.json` and `site/bun.lock`; CI actions by commit SHA.
- **LSP plugins** join with their language: `rust-analyzer-lsp` (`just rust install` puts `rust-analyzer` on PATH) for `installer/`; `pyright-lsp` (with `pyright-langserver` in `just setup`) arrives with the first Python code. No `typescript-lsp`: the site's few TypeScript files are covered by `astro check`.
- **Other tools** (rclone) join `just setup` with the first task that needs them; podman for image builds is a host prerequisite (`docs/setup.md`).
- **When to use what:** graphify, then Serena, for this repo and earlier work; context7 for library APIs. rtk wraps shell output on its own.
- **Gotchas:** rtk reshapes output (summaries, hidden lines), so run `rtk bypass <cmd>` where a script parses it. In auto mode, a command rtk's hook rewrote can come back unreviewed: send it again unchanged, once. `just claude plugins` skips a marketplace that the operator's user or managed settings list from another source (Claude Code refuses it) and says so. In the Mac's Fedora runner VM, SELinux keeps unprivileged containers off bind mounts and `/dev/vhost-vsock` (`just arch packages` and `just arch test` run with `--security-opt label=disable`); pacman's Landlock sandbox fails in unprivileged rootless podman (`podman build` steps use `--disable-sandbox`).

## Site
- `site/`: Astro with Starlight docs at `/docs` (`src/content.config.ts` prefixes ids with `docs/`); marketing pages in `src/pages/` at `/`. Bun only, never npm. ESLint, Prettier and `astro check`. English only, no i18n: UI strings live in the components; `src/content/i18n/en.json` is `{}` only because Starlight's core always reads that collection, and Astro warns when it is missing. Recipes in `site/justfile`.
  - It publishes `llms.txt` (`starlight-llms-txt`; `llms-small.txt` leaves out the prompt trail) and a `.md` twin of every docs page (`src/pages/[...slug].md.ts`).
  - Fonts and colours come from the branding file.
  - Sidebar groups: User Guide (`user/`) and Prompt Trail (`prompt-trail/`); add a group to `astro.config.mjs` with its first page. The site carries no developer guide.
- Prompt trail: one entry per task PR, written from `.claude/skills/s-build/entry-template.md`. A task on both bases goes to `site/src/content/docs/prompt-trail/<NNN>-<slug>.md`; a single-base task goes under `prompt-trail/arch/` or `prompt-trail/fedora/`, labelled Arch and Fedora by `src/routeData.ts`. Frontmatter `pr:` and `task:` draw the links under the title.
- Screenshots: `site/src/assets/prompt-trail/<NNN>/<name>.webp`, shown only by their own entry and made with `just site screenshot <raw.png> <out.webp>` (WebP, longest side 1600 px or less, 300 KB or less, metadata stripped).
  - `just site screenshots-check` (pre-commit) enforces it.
  - Site pages are never screenshotted.
  - Every screen a task builds or changes appears in its entry as a real screenshot, never as a text drawing; `just arch screenshots` captures the installer's in a VM.
- Docs entry: every task adds or updates the docs its change needs: user docs in `site/src/content/docs/user/` for what users see, developer notes in `docs/` for how to build and test it.

## Builds
- Arch x86_64 builds pin one Arch Linux Archive date, moved forward only through a test gate; Arch builds and tests run in a pinned container. Outputs go to `.build/` (gitignored).
- The Arch builder is `arch/Containerfile`, pinned with the archive date in `arch/vars.just`; mkarchiso runs in it as the unprivileged user `build` (rootless podman cannot mount devtmpfs).
- The host's architecture picks the Arch image (no cross-builds): Arch Linux ARM aarch64 builds natively in the Mac's runner VM from the same Containerfile and profile, on `docker.io/menci/archlinuxarm` by digest (no official image), with packages from ALARM's live repos and archiso and edk2-aarch64 from the archive date.
- Arch Linux ARM and Fedora build from their live repos; every image records its full package manifest.
- The Fedora builder is `fedora/Containerfile` (kiwi, QEMU with OVMF or AAVMF, socat, mock), its base pinned by digest in `fedora/vars.just` (a manifest list, so one pin serves both architectures); kiwi runs as container root in rootless `--privileged` podman, with its work dir on a tmpfs container root mounts itself, because SELinux labels cannot be set on podman's overlay, nor on podman's `--tmpfs` on an SELinux host (mounted with one fixed `context=`).
- The host's architecture picks the Fedora image too: aarch64 builds natively in the Mac's runner VM from the same `fedora/iso/` description, where `arch="…"` on a `<package>` in `config.kiwi` keeps it to one architecture (GRUB, x86 microcode).
- Fedora images never ship `fedora-release*`, `fedora-logos` or `fedora-release-notes` (Fedora's trademark guidelines); our Fedora `stackific-release` replaces `generic-release` and keeps `generic-release-common` (Fedora's rpm macros, dnf5 defaults, the `/etc` symlinks), and `generic-logos` provides `system-logos`.
- Our packages: makepkg in a fresh pinned container on Arch, mock on Fedora. They hold configuration, artwork, release files and third-party tools we package (Shelly on Arch).
  - Arch: `arch/packages/<name>/PKGBUILD`, base-neutral sources in `desktop/` and `branding/`, Arch-only sources next to the PKGBUILD (staged by name like the rest); `just arch packages` builds them into the local repo `.build/arch/repo/`, which the ISO build reads as `[stackific]` (unsigned until release signing).
  - Fedora: `fedora/packages/<name>/<name>.spec`, the same sources and install paths as the PKGBUILDs (Fedora-only sources beside the spec); mock runs as container root with `--isolation=simple --no-bootstrap-image`, and `just fedora packages` builds the RPMs into the local repo `.build/fedora/repo/` (createrepo_c, unsigned until release signing). `stackific-installer` alone builds with `mock --enable-network` (`cargo fetch` in `%prep`).
- The ArtCraft apps come from upstream's own release packages (`.pkg.tar.zst`, `.rpm`), checked against its `SHA256SUMS.txt` and pinned by version.
- Fedora takes MangoWM, noctalia-greeter and Limine from Terra; the images carry dnf5 (installed systems update with it) but no Terra repo file (installed systems get Terra's packages from our repos, task 028); scenefx and Noctalia come from Fedora. `fedora/iso/terra-repo.sh` limits Terra with `includepkgs`, and packages and metadata are checked against Terra's key committed as `fedora/iso/terra-45.asc`. NVIDIA and Broadcom wl come from RPM Fusion; Bazaar from Flathub.
- Signing: packages, repo databases, `SHA256SUMS` and images use a subkey of an offline project key; the subkey lives only in the `release` environment.

## CI and releases
- GitHub Actions in `.github/workflows/`; every step after checkout and just's setup calls a just recipe. `check` (pull requests, pushes to `main`) runs `just lefthook all` and `just gitleaks history`.
- GitHub-hosted Linux runners run checks and container tests.
- The operator's Linux PC runs the self-hosted runner for x86_64 images and their VM tests (labels `self-hosted`, `Linux`, `X64`); `just runner setup` makes a machine a runner, never `just setup`.
- The operator's Mac runs the self-hosted runner for aarch64 images in an arm64 Linux VM (labels `self-hosted`, `Linux`, `ARM64`); `just runner setup` on the Mac creates the VM (`just runner vm`) and runs itself inside it. The VM runs the Fedora images' release (Fedora 45, its Cloud image pinned in `just/runner/vars.just`), because the VM enforces SELinux and a rootless image build can set only the SELinux contexts the VM's own policy knows.
- `images` runs on every push to `main` (a merged PR) on both the PC and the Mac, and by hand (`workflow_dispatch`) (operator 2026-11-08). On a push it builds only the images whose files changed (`just release changed`; keep its path lists current when an image gains sources); by hand it builds all four. It publishes the images of a merged pull request's commit as the test release `v0.0.0-pr.<n>` (a prerelease, replaced on a rerun; not a release).
- Self-hosted runners take only `push` to `main` and `workflow_dispatch`, never `pull_request`; `just runner register` makes every outside contributor's run wait for approval.
- `release` runs only by hand (`workflow_dispatch`, input `version`):
  - refuses a version that is not semver;
  - builds and tests all four images, then signs them;
  - tags `v<version>` and creates the GitHub release with each ISO, `SHA256SUMS` and its signature attached;
  - today: all four images, the x86_64 ones on the PC and the aarch64 ones on the Mac, unsigned; signing joins when it exists (`just release create`, `upload`, `publish`: a draft until every image is attached);
  - an ISO over 2 GiB goes to Cloudflare R2 instead and the release links it: secrets `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, variable `R2_BUCKET` (the bucket name, such as `stackific-os`). Wired only once an ISO needs it.
  - A released version never changes.
- `release/notes.md` opens the notes of every release and test build; GitHub's generated notes start at the newest full release (`--notes-start-tag`), because GitHub would otherwise start them at the newest test build.
- Secrets are never printed and never committed; gitleaks runs in pre-commit and in `check`.

## Tests
- Tests run without asking; anything that needs a person gets step-by-step instructions for the operator.
- As much automation as is pragmatic:
  - static checks on every commit;
  - container tests for packages, package lists and headless MangoWM with Noctalia;
  - QEMU UEFI (OVMF, AAVMF) boot, session and unattended-install tests for every image, in both install modes, on blank and Windows-like disk images.
- VMs get test access only from one-boot SMBIOS credentials with a token-gated vsock shell; no image ships sshd or test units.
- Boot tests reach the VM only through the vsock shell from `systemd.extra-unit.*`/`systemd.unit-dropin.*` credentials and wait with `systemctl is-system-running --wait` over it; `vmm.notify_socket` over vsock drops messages, so it is not used (`tests/vm/`).
- `tests/vm/boot.sh <iso> <out-dir> <checks>` runs the checks every image passes, then sources the image's check file: `tests/vm/<base>.sh` when every image of the base passes the same checks, `tests/vm/<base>-<arch>.sh` when an architecture needs its own (it sources `<base>.sh` first; none does since task 018); a check that only one image passes goes there. `boot.sh` sets `efi` (Limine's `BOOTX64.EFI` or `BOOTAA64.EFI`) for the check files.
- `boot.sh -` boots the attached disk alone (an installed system) with an earlier run's firmware variables (`OVMF_VARS=<out>/vars.fd`, which every run leaves) and saves Limine's menu as `limine.png`; `just arch test-install` checks the booted install with `tests/vm/installed.sh`.
- The test shell is a `systemd-socket-activate` service (`tests/vm/stackific-test-shell.service`), not a socket unit, because Fedora's SELinux policy denies PID 1 a listening vsock socket labelled for a shell.
- Over the test shell on Fedora, `systemctl --user -M <user>@` fails (`systemd-run -P` hands the shell's vsock socket to PID 1 and dbus-broker, which SELinux silently denies); checks run `systemctl --user` as the user through `runuser`.
- Tests log into GNOME through noctalia-greeter (key presses), never greetd's `[initial_session]`: that opens a logind session of type `tty`, and GNOME Shell starts only in a `wayland` one.
- Installed-system boots unlock with systemd's `cryptsetup.passphrase` SMBIOS credential (`STACKIFIC_TEST_PASSPHRASE`); one last boot types at the prompt (`tests/vm/unlock.sh`, `PREBOOT` in `boot.sh`): three wrong passphrases, the boot-failure screen, then the password on the German layout. The password is `test_password` in `tests/vars.just`.
- Install tests next to Windows use real BitLocker volumes from cryptsetup's test images (pinned in `tests/vars.just`, downloaded by `boot.sh`'s `asset`, sent to the guest with `vm_put`) and a fake Windows (Limine's program as `bootmgfw.efi` plus a "Windows Boot Manager" entry), which the installed system starts through both Limine entries.
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
- The operator does every git operation; skills may read history but never commit, push, rebase or open PRs. A task's Done box is ticked once its PR merges: an agent that finds a merged PR (`gh pr list --state merged`) whose task is unticked ticks it in a chore commit.

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
