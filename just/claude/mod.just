# Claude Code module: installs the marketplaces and plugins that .claude/settings.json declares, at project scope.
# node, which the plugins' hooks need anyway, reads the JSON.

# Claude Code refuses to add a marketplace whose source differs from an entry of the same name in user or managed
# settings; such a marketplace is skipped with a warning and its plugins install from the source that entry lists.
# Add each marketplace (at its pinned ref) and install each plugin that .claude/settings.json declares
plugins:
  #!/usr/bin/env bash
  set -euo pipefail
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"; case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac # setup put it in the shell profiles; this shell may predate that
  cd "{{ justfile_directory() }}"
  command -v claude >/dev/null || { echo "claude CLI not found; install Claude Code first." >&2; exit 1; }
  command -v node >/dev/null || { echo "node not found; install nodejs (plugin hooks run on it)." >&2; exit 1; }
  # settings.json is the single source of truth: read the marketplaces and plugins from it
  node -e 'const fs = require("fs");
    const read = (f) => { try { return JSON.parse(fs.readFileSync(f, "utf8")).extraKnownMarketplaces ?? {}; } catch { return {}; } };
    const managed = process.platform === "darwin" ? "/Library/Application Support/ClaudeCode/managed-settings.json" : "/etc/claude-code/managed-settings.json";
    const user = process.env.CLAUDE_CONFIG_DIR ?? require("os").homedir() + "/.claude";
    const theirs = { ...read(user + "/settings.json"), ...read(managed) };
    const key = (s) => [s.source, s.repo ?? s.url ?? s.path, s.ref].filter(Boolean).join(" ");
    const ours = { "claude-plugins-official": { scope: "user", source: { source: "github", repo: "anthropics/claude-plugins-official" } } };
    for (const [n, m] of Object.entries(require("./.claude/settings.json").extraKnownMarketplaces ?? {})) ours[n] = { scope: "project", ...m };
    for (const [n, m] of Object.entries(ours)) {
      if (theirs[n] && key(theirs[n].source) !== key(m.source)) {
        console.error(`Skipping marketplace ${n}: user or managed settings list it as "${key(theirs[n].source)}"; remove that entry to use the pinned "${key(m.source)}".`);
        continue;
      }
      console.log("m " + m.scope + " " + m.source.repo + (m.source.ref ? "#" + m.source.ref : ""));
    }
    for (const [p, on] of Object.entries(require("./.claude/settings.json").enabledPlugins ?? {})) if (on) console.log("p project " + p);' |
  while read -r kind scope name; do
    if [ "$kind" = m ]; then claude plugin marketplace add --scope "$scope" "$name"; else claude plugin install --scope "$scope" "$name"; fi
  done

# Report whether the claude CLI and node (plugin hooks) are there and every declared plugin is installed for this repo
check:
  #!/usr/bin/env bash
  set -euo pipefail
  d="${XDG_BIN_HOME:-$HOME/.local/bin}"; case ":$PATH:" in *":$d:"*) ;; *) PATH="$d:$PATH" ;; esac # setup put it in the shell profiles; this shell may predate that
  cd "{{ justfile_directory() }}"
  command -v claude >/dev/null || { echo "claude: CLI missing (install Claude Code)"; exit 1; }
  command -v node >/dev/null || { echo "claude: node missing; ponytail's and caveman's hooks run on it (install nodejs)"; exit 1; }
  claude plugin list --json | node -e 'const want = Object.entries(require("./.claude/settings.json").enabledPlugins ?? {}).filter(([, on]) => on).map(([p]) => p);
    const have = new Set(JSON.parse(require("fs").readFileSync(0, "utf8"))
      .filter((e) => e.scope !== "project" || e.projectPath === process.cwd()).map((e) => e.id));
    const missing = want.filter((p) => !have.has(p));
    console.log(missing.length ? `claude: missing plugins ${missing.join(", ")} (just claude plugins)` : `claude: ok (${want.length} plugins)`);
    process.exit(missing.length ? 1 : 0);'
