/**
 * Turns a raw screenshot (PNG, JPEG, WebP…) into a prompt-trail image: auto-oriented, scaled down
 * so its longest side is at most `maxSide` (never up), metadata stripped (sharp drops EXIF, ICC
 * and XMP unless asked to keep them) and encoded as WebP. It encodes both a lossy (quality 80,
 * smart chroma subsampling, so coloured text keeps its edges) and a lossless WebP and keeps the
 * smaller: flat screens (terminals, consoles, docs pages) often come out smaller lossless, with
 * text exactly as captured, while wallpapers and photos need lossy.
 *
 * Prints the input and output sizes; exits 1 when the output is still over `maxBytes`, since
 * `check.ts` would refuse it (crop the capture to the part that matters).
 *
 * Usage: bun screenshots/process.ts <input> <output.webp>  (run by `just site screenshot`)
 */
import { mkdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';
import sharp from 'sharp';
import { maxBytes, maxSide } from './limits';

/** A human-readable size, in KB. */
const kb = (bytes: number) => `${(bytes / 1024).toFixed(1)} KB`;

const [input, output] = process.argv.slice(2);
if (!input || !output?.endsWith('.webp')) {
  console.error('usage: process.ts <input> <output.webp>');
  process.exit(2);
}

// rotate() with no angle applies the EXIF orientation before the metadata is dropped
const scaled = await sharp(input)
  .rotate()
  .resize({ width: maxSide, height: maxSide, fit: 'inside', withoutEnlargement: true })
  .toBuffer();
const [lossy, lossless] = await Promise.all([
  sharp(scaled).webp({ quality: 80, effort: 6, smartSubsample: true }).toBuffer({ resolveWithObject: true }),
  sharp(scaled).webp({ lossless: true, effort: 6 }).toBuffer({ resolveWithObject: true }),
]);
const best = lossless.data.length <= lossy.data.length ? lossless : lossy;

mkdirSync(dirname(output), { recursive: true });
writeFileSync(output, best.data);

const before = await sharp(input).metadata();
console.log(`${input}: ${before.width}×${before.height}, ${kb(statSync(input).size)}`);
console.log(
  `${output}: ${best.info.width}×${best.info.height}, ${kb(best.data.length)} (${best === lossless ? 'lossless' : 'lossy'})`,
);
if (best.data.length > maxBytes) {
  console.error(`${output} is over ${kb(maxBytes)}: crop the capture to the part that matters`);
  process.exit(1);
}
