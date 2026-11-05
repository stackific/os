# Shared uv settings. Import this from any module that installs or runs Python tools through uv.

# Pinned uv release (github.com/astral-sh/uv) and the SHA-256 of its release asset per host
uv_version := "0.13.5"
uv_target := if os() == "macos" { arch() + "-apple-darwin" } else { arch() + "-unknown-linux-gnu" }
uv_sha256 := if uv_target == "aarch64-apple-darwin" { "743f85035a1996ecb350659e01cd9faea9709829be09b1b4b03c73eaf3c9cd73" } else if uv_target == "aarch64-unknown-linux-gnu" { "4bb69cf95ed13f23713bb7bc09f241896ea0c75b7c557e615a20604b82f9c403" } else { "f68d291c75783122dad58ab77def0e13f3233f358731db5d6d7e8715603b928f" }

# Python every uv tool runs on, downloaded by uv (never the host's)
python_version := "3.13"

# Packages uploaded after this moment are ignored, so tool installs resolve the same on every machine.
# Move it, deliberately, when a pin needs something newer.
uv_exclude_newer := "2026-11-04T00:00:00Z"

uv_dir := justfile_directory() / ".tools" / "uv"
uv := uv_dir / "bin" / "uv"

# Executables of uv tools (graphify, serena); launchers on PATH point here
tools_bin := justfile_directory() / ".tools" / "bin"

# Keeps everything uv writes (cache, Pythons, tools) under .tools/ and ignores user and project uv config
uv_env := "env UV_CACHE_DIR='" + uv_dir / "cache" + "' UV_PYTHON_INSTALL_DIR='" + uv_dir / "python" + "' UV_PYTHON_INSTALL_BIN=0 UV_PYTHON_PREFERENCE=only-managed UV_TOOL_DIR='" + uv_dir / "tools" + "' UV_TOOL_BIN_DIR='" + tools_bin + "' UV_NO_CONFIG=1 UV_EXCLUDE_NEWER=" + uv_exclude_newer

# Installs a pinned uv tool: `{{ uv_tool_install }} pkg==ver`
uv_tool_install := uv_env + " '" + uv + "' tool install --quiet --python " + python_version
