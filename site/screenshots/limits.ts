/**
 * Limits every prompt-trail screenshot meets, shared by `process.ts` (which produces them) and
 * `check.ts` (which enforces them before a commit).
 */

/** Longest side of a published screenshot, in pixels. */
export const maxSide = 1600;

/** Largest published screenshot, in bytes (300 KB). */
export const maxBytes = 300 * 1024;

/**
 * Where entry screenshots live, relative to `site/`: one folder per entry, named by the entry's
 * number, as in `src/assets/prompt-trail/<NNN>/<name>.webp`.
 */
export const assetsDir = 'src/assets/prompt-trail';

/**
 * Where prompt-trail entries live, relative to `site/`: `<docsDir>/<NNN>-<slug>.md` for a task on both
 * bases, `<docsDir>/<base>/<NNN>-<slug>.md` for a single-base one.
 */
export const docsDir = 'src/content/docs/prompt-trail';

/** Folders under `docsDir` that hold single-base entries. */
export const baseDirs = ['arch', 'fedora'];
