# Fetch module: installs one executable from a pinned release download into .tools/, checked against its SHA-256.

import '../host/vars.just'

# Install <member> of the archive at <url> as <dest>, after checking the download's SHA-256 against <sha256>.
# <member> is the executable's path inside a .zip or .tar.gz, or empty when the download is the executable.
# A no-op while <dest>.sha256 records <sha256>, so a pin bump (a new checksum) installs the new release.
get url sha256 dest member="":
  #!/usr/bin/env bash
  set -euo pipefail
  stamp="{{ dest }}.sha256"
  if [ -x "{{ dest }}" ] && [ "$(cat "$stamp" 2>/dev/null)" = "{{ sha256 }}" ]; then exit 0; fi
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  file="$tmp/$(basename "{{ url }}")"
  # Retries: a release host answers 502, 503 or 504 now and then (rclone: HTTP 504 in an images run, 2026-11-09)
  curl -fsSL --retry 5 --retry-delay 10 -o "$file" "{{ url }}"
  echo "{{ sha256 }}  $file" | {{ sha256_check }}
  mkdir "$tmp/x"
  case "$file" in
    *.zip) unzip -q "$file" -d "$tmp/x" ;;
    *.tar.gz) tar -xzf "$file" -C "$tmp/x" ;;
    *) mv "$file" "$tmp/x/" ;;
  esac
  mkdir -p "$(dirname "{{ dest }}")"
  install -m755 "$tmp/x/{{ if member == "" { file_name(url) } else { member } }}" "{{ dest }}"
  echo "{{ sha256 }}" > "$stamp"
