# Claude Code module: installs the marketplaces and plugins that .claude/settings.json declares, at project scope.
# node, which the plugins' hooks need anyway, reads the JSON.

# Add each marketplace (at its pinned ref) and install each plugin that .claude/settings.json declares
plugins:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ justfile_directory() }}"
  command -v claude >/dev/null || { echo "claude CLI not found; install Claude Code first." >&2; exit 1; }
  command -v node >/dev/null || { echo "node not found; install nodejs (plugin hooks run on it)." >&2; exit 1; }
  claude plugin marketplace add anthropics/claude-plugins-official > /dev/null
  # settings.json is the single source of truth: read the marketplaces and plugins from it
  node -e 'const s = require("./.claude/settings.json");
    for (const m of Object.values(s.extraKnownMarketplaces ?? {})) console.log("m " + m.source.repo + (m.source.ref ? "#" + m.source.ref : ""));
    for (const [p, on] of Object.entries(s.enabledPlugins ?? {})) if (on) console.log("p " + p);' |
  while read -r kind name; do
    if [ "$kind" = m ]; then claude plugin marketplace add --scope project "$name"; else claude plugin install --scope project "$name"; fi
  done

# Report whether the claude CLI and node (plugin hooks) are there and every declared plugin is installed for this repo
check:
  #!/usr/bin/env bash
  set -euo pipefail
  cd "{{ justfile_directory() }}"
  command -v claude >/dev/null || { echo "claude: CLI missing (install Claude Code)"; exit 1; }
  command -v node >/dev/null || { echo "claude: node missing; ponytail's and caveman's hooks run on it (install nodejs)"; exit 1; }
  claude plugin list --json | node -e 'const want = Object.entries(require("./.claude/settings.json").enabledPlugins ?? {}).filter(([, on]) => on).map(([p]) => p);
    const have = new Set(JSON.parse(require("fs").readFileSync(0, "utf8"))
      .filter((e) => e.scope !== "project" || e.projectPath === process.cwd()).map((e) => e.id));
    const missing = want.filter((p) => !have.has(p));
    console.log(missing.length ? `claude: missing plugins ${missing.join(", ")} (just claude plugins)` : `claude: ok (${want.length} plugins)`);
    process.exit(missing.length ? 1 : 0);'
