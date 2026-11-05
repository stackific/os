/**
 * Markdown twin of every docs page, built as a static file next to the page:
 * /docs/user-guide/installation/ → /docs/user-guide/installation.md.
 * Gives LLMs and agents clean source for any page.
 */
import type { APIRoute, GetStaticPaths, ImageMetadata } from 'astro';
import { getImage } from 'astro:assets';
import { getCollection, type CollectionEntry } from 'astro:content';
import { posix } from 'node:path';
import { REPO_URL } from '../repo';

export const getStaticPaths = (async () =>
  (await getCollection('docs')).map((doc) => ({ params: { slug: doc.id }, props: { doc } }))) satisfies GetStaticPaths;

/** Every image under src/assets, keyed by its path relative to the site root (`src/assets/…`). */
const assets = import.meta.glob<{ default: ImageMetadata }>('/src/assets/**/*.{png,jpg,jpeg,webp,avif,gif}', {
  eager: true,
});

/**
 * Points the page's relative Markdown images (`![alt](../../assets/x.webp)`, which astro:assets
 * resolves for the HTML page) at the same built image, since the twin is served from another URL.
 */
async function resolveImages(body: string, filePath: string | undefined): Promise<string> {
  if (!filePath) return body;
  const dir = posix.dirname(filePath);
  const urls = new Map<string, string>();
  for (const [, target] of body.matchAll(/!\[[^\]]*\]\((\.{1,2}\/[^)\s]+)/g)) {
    const image = assets['/' + posix.join(dir, target)];
    if (image) urls.set(target, (await getImage({ src: image.default })).src);
  }
  return body.replace(/(!\[[^\]]*\]\()(\.{1,2}\/[^)\s]+)/g, (match, head, target) =>
    urls.has(target) ? head + urls.get(target) : match,
  );
}

/** Serves the page source with its title as a heading (frontmatter dropped; `pr:` and `task:` links kept under the title). */
export const GET: APIRoute<{ doc: CollectionEntry<'docs'> }> = async ({ props: { doc } }) => {
  const links = [
    doc.data.pr && `[Pull request](${doc.data.pr})`,
    doc.data.task && `[Task](${REPO_URL}/blob/main/${doc.data.task})`,
  ].filter(Boolean);
  const head = links.length ? `${links.join(' · ')}\n\n` : '';
  return new Response(`# ${doc.data.title}\n\n${head}${await resolveImages(doc.body ?? '', doc.filePath)}`, {
    headers: { 'Content-Type': 'text/markdown; charset=utf-8' },
  });
};
