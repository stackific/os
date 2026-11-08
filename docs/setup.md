# Developer setup

How to set up a machine to build, test and change Stackific OS. Rules for all work are in [AGENTS.md](../AGENTS.md).

## Setup

Works on Linux (x86_64 or aarch64) and on macOS with Apple silicon. Building and testing images needs Linux.

1. Install the host prerequisites. Everything else comes from `just setup`.
   - Arch: `sudo pacman -S --needed git just nodejs npm unzip`
   - Fedora: `sudo dnf install git just nodejs npm unzip`
   - macOS: `xcode-select --install`, then `brew install just node lima`. Lima runs the Fedora VM that builds and tests the aarch64 images; its nested virtualization, which gives the VM `/dev/kvm`, needs Apple M3 or later and macOS 15 or later.
   - Linux also needs rootless `podman` for image builds (Arch: `sudo pacman -S --needed podman`, Fedora: `sudo dnf install podman`), and read-write `/dev/kvm` and `/dev/vhost-vsock`. Arch and Fedora make both 0666; elsewhere, add yourself to the `kvm` group (`sudo usermod -aG kvm "$USER"`, then log in again). Check with `ls -l /dev/kvm /dev/vhost-vsock`. `just doctor` reports them on its `arch:` line and fails without them.
2. Install Claude Code: `curl -fsSL https://claude.ai/install.sh | bash`
3. Run `just setup`. It installs the pinned tools, the site's packages and the git hooks, then runs `just doctor`. On a Mac it also creates or starts the Fedora 45 VM `stackific-runner` (`just runner vm`, under "Mac (aarch64 images)" below) and clones the repo into it, without making it a runner. `just doctor` checks the VM's Fedora release on its `runner vm:` line.
4. Open a new shell.
5. Start Claude Code in the repo with `claude`.
   - Execute `/reload-plugins`
   - Connect the `serena` MCP server and the project's plugins when it asks.

## Everyday use

- `just` lists every recipe. `just doctor` rechecks the tools.
- `just graphify update` rebuilds the code graph after code changes.
- `just site dev` serves the site at `localhost:4321`.
- `just arch iso` builds the Arch live ISO for the host's architecture into `.build/arch/<arch>/`: x86_64 on a Linux PC, aarch64 in the Mac's runner VM. `just arch test` boots it under QEMU and checks it. See [arch-iso.md](arch-iso.md); [checks.md](checks.md) lists the checks.
- `just arch packages` builds our Arch packages into `.build/arch/repo/` (`just arch iso` runs it first).
- `just installer test`, `just installer lint` and `just installer demo` test, lint and show the installer on any machine, with the Rust toolchain `just setup` puts in `.tools/rust/`; `just arch test-install` installs it in a VM. See [installer.md](installer.md).
- `just lefthook all` runs every pre-commit check on every tracked file.
- Distro work runs through `/s-task <ask>`, then `/s-build <task>`. See [AGENTS.md](../AGENTS.md).

## Self-hosted runners

The `images` workflow builds and boot-tests the images on our own machines when started by hand from the Actions tab (for now; merges to `main` no longer start it, `docs/releases.md`). The x86_64 images run on a Linux PC and the aarch64 images on a Mac. A machine becomes a runner only through `just runner setup`; `just setup` never does it.

The runner runs every job as the user who set it up. The workflow never runs on pull requests, and `just runner register` makes GitHub hold every outside contributor's workflow run until a maintainer approves it. Never add `pull_request` to a workflow that runs on `self-hosted`.

### Linux PC (x86_64 images)

1. Finish the setup above, including podman and the `/dev/kvm` and `/dev/vhost-vsock` access.
2. Log the pinned gh in to GitHub as a repo admin: `just gh login`.
3. Run `just runner setup`. It installs the runner into `.tools/actions-runner/` and registers this PC with the labels `self-hosted`, `Linux` and `X64`. It then installs and starts the runner as a systemd service under your user, which asks for your sudo password once.
4. Check that it is online: `just runner status` prints `<hostname> online busy=false self-hosted,Linux,X64`.

The service starts at boot. If `just runner status` shows the PC offline, restart the service (the runner's `svc.sh` names it after the repo and this machine's hostname), then check again:

```sh
sudo systemctl restart "actions.runner.stackific-os.$(hostname).service"
just runner status
journalctl -u "actions.runner.stackific-os.$(hostname).service" -e   # its log, when it stays offline
```

To take the PC out of CI, run `just runner remove`.

### Mac (aarch64 images)

The runner runs in an arm64 Linux VM, because image builds need Linux. The VM's nested virtualization, which gives it `/dev/kvm`, needs Apple M3 or later and macOS 15 or later.

1. Finish the setup above, Lima included.
2. Run `just runner setup` in this repo on the Mac.
   - Like `just setup`, it creates or starts the Fedora 45 VM `stackific-runner` (6 CPUs, 16 GiB memory, 100 GiB disk). A VM made with less memory grows with `limactl stop stackific-runner`, `limactl edit --memory 16 stackific-runner`, then `just runner vm`. The VM mounts nothing from the Mac. The recipe installs what the runner needs in it (git, just, podman, libicu) and clones the repo to `~/os` in it. The VM's Fedora release must be the Fedora images' release, because the Fedora image build can set only the SELinux labels the VM's own policy knows.
   - It then runs `just runner setup` inside the VM. gh asks you to log in as a repo admin, and the VM registers with the labels `self-hosted`, `Linux` and `ARM64` and runs the runner as a service.
3. Check that it is online: `just runner status` prints `<hostname> online busy=false self-hosted,Linux,ARM64`.

After the Mac restarts, run `just runner vm` again; the runner service starts with the VM.

A VM made on an older Fedora release moves up in place, inside the VM (`limactl shell stackific-runner`): `sudo dnf upgrade --refresh`, then `sudo dnf system-upgrade download --releasever=45`, then `sudo dnf offline reboot`. The VM applies the upgrade and boots into Fedora 45 by itself, runner service included; `just runner check` then prints `runner vm: ok (stackific-runner, Fedora 45)`.
