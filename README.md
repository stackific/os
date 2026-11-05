# Stackific OS

A Linux distribution on two bases, Arch Linux and Fedora 45, for x86_64 and aarch64. Rules for all work are in [AGENTS.md](AGENTS.md).

## Developer setup

Works on Linux (x86_64 or aarch64) and on macOS with Apple silicon. Building and testing images needs Linux.

1. Install the host prerequisites. Everything else comes from `just setup`.
   - Arch: `sudo pacman -S --needed git just nodejs npm unzip`
   - Fedora: `sudo dnf install git just nodejs npm unzip`
   - macOS: `xcode-select --install`, then `brew install just node`
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
- `just lefthook all` runs every pre-commit check on every tracked file.
- Distro work runs through `/s-task <ask>`, then `/s-build <task>`. See [AGENTS.md](AGENTS.md).
