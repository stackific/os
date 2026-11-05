/**
 * Checks the prompt-trail screenshots under `src/assets/prompt-trail/<NNN>/`:
 *
 * - every file there is `<NNN>/<name>.webp` (lowercase words joined by hyphens), at most
 *   `maxSide` pixels on its longest side and at most `maxBytes` bytes, as `process.ts` makes them;
 * - every file there is shown by entry `<NNN>-*.md` (at the top of the trail or in a base's folder), so none is left behind;
 * - every image an entry shows from the site's own files is one of these, in the entry's own
 *   folder, with alt text (remote images are not allowed: the repo holds what the trail shows).
 *
 * Exits 1 and prints every problem when any is found.
 *
 * Usage: bun screenshots/check.ts  (run inside site/ by `just site screenshots-check`)
 */
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, normalize, relative } from 'node:path';
import sharp from 'sharp';
import { assetsDir, baseDirs, docsDir, maxBytes, maxSide } from './limits';

const problems: string[] = [];

/** Every file under `dir`, as paths that start with `dir`. */
const files = (dir: string): string[] =>
  existsSync(dir)
    ? readdirSync(dir, { recursive: true, encoding: 'utf8' })
        .map((p) => join(dir, p))
        .filter((p) => statSync(p).isFile())
    : [];

/** Markdown images (`![alt](target "title")`) in an entry, outside code blocks. */
function images(markdown: string): { alt: string; target: string }[] {
  const prose = markdown.replace(/^ {0,3}(`{3,}|~{3,})[^\n]*\n[\s\S]*?^ {0,3}\1\s*$/gm, '');
  return [...prose.matchAll(/!\[([^\]]*)\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)/g)].map((m) => ({
    alt: m[1].trim(),
    target: m[2],
  }));
}

// Images the entries show, keyed by normalized path relative to site/
const shown = new Set<string>();
for (const entries of [docsDir, ...baseDirs.map((b) => join(docsDir, b))]) {
  for (const name of existsSync(entries) ? readdirSync(entries).filter((n) => /^\d{3}-.*\.mdx?$/.test(n)) : []) {
    const entry = join(entries, name);
    const own = join(assetsDir, name.slice(0, 3));
    for (const { alt, target } of images(readFileSync(entry, 'utf8'))) {
      if (/^[a-z]+:/i.test(target) || target.startsWith('//')) {
        problems.push(`${entry}: ${target} is remote; process it into ${own}/ with \`just site screenshot\``);
        continue;
      }
      const path = normalize(join(entries, target));
      if (!alt) problems.push(`${entry}: ${target} has no alt text`);
      if (!path.startsWith(own + '/')) problems.push(`${entry}: ${target} is not in ${own}/`);
      else if (!existsSync(path)) problems.push(`${entry}: ${target} does not exist`);
      else shown.add(path);
    }
  }
}

for (const file of files(assetsDir)) {
  const name = relative(assetsDir, file);
  if (!/^\d{3}\/[a-z0-9]+(-[a-z0-9]+)*\.webp$/.test(name)) {
    problems.push(`${file}: must be <NNN>/<lowercase-hyphenated-name>.webp`);
    continue;
  }
  const bytes = statSync(file).size;
  if (bytes > maxBytes) problems.push(`${file}: ${Math.ceil(bytes / 1024)} KB is over ${maxBytes / 1024} KB`);
  const { format, width = 0, height = 0 } = await sharp(file).metadata();
  if (format !== 'webp') problems.push(`${file}: is ${format}, not WebP`);
  if (Math.max(width, height) > maxSide) problems.push(`${file}: ${width}×${height} is over ${maxSide} px`);
  if (!shown.has(file)) problems.push(`${file}: no prompt-trail entry ${name.slice(0, 3)}-*.md shows it`);
}

if (problems.length > 0) {
  console.error(problems.join('\n'));
  process.exit(1);
}
