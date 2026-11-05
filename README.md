# Stackific OS

A Linux distribution on two bases, Arch Linux and Fedora 45, for x86_64 and aarch64. Rules for all work are in [AGENTS.md](AGENTS.md).

## Developer setup

Works on Linux (x86_64 or aarch64) and on macOS with Apple silicon. Building and testing images needs Linux.

1. Install the host prerequisites. Everything else comes from `just setup`.
   - Arch: `sudo pacman -S --needed git just nodejs npm unzip`
   - Fedora: `sudo dnf install git just nodejs npm unzip`
   - macOS: `xcode-select --install`, then `brew install just node`
   - Linux also needs rootless `podman` for image builds (Arch: `sudo pacman -S --needed podman`, Fedora: `sudo dnf install podman`), and read-write `/dev/kvm` and `/dev/vhost-vsock`. Arch and Fedora make both 0666; elsewhere, add yourself to the `kvm` group (`sudo usermod -aG kvm "$USER"`, then log in again). Check with `ls -l /dev/kvm /dev/vhost-vsock`. `just doctor` reports them on its `arch:` line and fails without them.
2. Install Claude Code: `curl -fsSL https://claude.ai/install.sh | bash`
3. Put `~/.local/bin` on your PATH, where the launchers for `rtk`, `graphify` and `claude` go. On macOS:
   ```sh
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zprofile
   ```
   Then open a new shell.
4. Run `just setup`. It installs the pinned tools, the site's packages and the git hooks, then runs `just doctor`.
   - If doctor names a file in `~/.local/bin` with the wrong version, that file isn't ours: remove it and run `just setup` again.
5. Start Claude Code in the repo with `claude`.
   - Execute `/reload-plugins`
   - Connect the `serena` MCP server and the project's plugins when it asks.

## Everyday use

- `just` lists every recipe. `just doctor` rechecks the tools.
- `just graphify update` rebuilds the code graph after code changes.
- `just site dev` serves the site at `localhost:4321`.
- `just arch iso` builds the Arch x86_64 live ISO into `.build/arch/x86_64/`; `just arch test` boots it under QEMU and checks it. See [docs/arch-iso.md](docs/arch-iso.md); [docs/checks.md](docs/checks.md) lists the checks.
- `just arch packages` builds our Arch packages into `.build/arch/repo/` (`just arch iso` runs it first).
- `just lefthook all` runs every pre-commit check on every tracked file.
- Distro work runs through `/s-task <ask>`, then `/s-build <task>`. See [AGENTS.md](AGENTS.md).

## Self-hosted runners

The `images` workflow builds and boot-tests the images on our own machines after each merge to `main`, or when started by hand from the Actions tab. The x86_64 images run on a Linux PC and the aarch64 images on a Mac. A machine becomes a runner only through `just runner setup`; `just setup` never does it.

The runner runs every job as the user who set it up. The workflow never runs on pull requests, and `just runner register` makes GitHub hold every outside contributor's workflow run until a maintainer approves it. Never add `pull_request` to a workflow that runs on `self-hosted`.

### Linux PC (x86_64 images)

1. Finish the developer setup above, including podman and the `/dev/kvm` and `/dev/vhost-vsock` access.
2. Log the pinned gh in to GitHub as a repo admin: `just gh login`.
3. Run `just runner setup`. It installs the runner into `.tools/actions-runner/` and registers this PC with the labels `self-hosted`, `Linux` and `X64`. It then installs and starts the runner as a systemd service under your user, which asks for your sudo password once.
4. Check that it is online: `just runner status` prints `<hostname> online busy=false self-hosted,Linux,X64`.

The service starts at boot. To take the PC out of CI, run `just runner remove`.

### Mac (aarch64 images)

The runner runs in an arm64 Linux VM, because image builds need Linux. The VM's nested virtualization, which gives it `/dev/kvm`, needs Apple M3 or later and macOS 15 or later.

1. Install Lima: `brew install lima`.
2. Run `just runner setup` in this repo on the Mac.
   - It creates and starts the Fedora VM `stackific-runner` (6 CPUs, 12 GiB memory, 100 GiB disk), which mounts nothing from the Mac, installs what the runner needs in it (git, just, podman, libicu) and clones the repo to `~/os` in it.
   - It then runs `just runner setup` inside the VM. gh asks you to log in as a repo admin, and the VM registers with the labels `self-hosted`, `Linux` and `ARM64` and runs the runner as a service.
3. Check that it is online: `just runner status` prints `<hostname> online busy=false self-hosted,Linux,ARM64`.

After the Mac restarts, run `just runner vm` again; the runner service starts with the VM.
