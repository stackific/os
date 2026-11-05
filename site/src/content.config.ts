/**
 * Content collections for the site.
 *
 * `docs` is Starlight's collection, loaded from src/content/docs as usual. Entry ids (and so URLs)
 * get a `docs/` prefix, so every Starlight page is served under /docs/*. That leaves the site root
 * free for regular Astro pages in src/pages.
 *
 * `i18n` exists only because Starlight's core always looks it up for UI string overrides, and Astro
 * warns about a missing or empty collection. Its one entry, src/content/i18n/en.json, is `{}`: the
 * site stays English-only with Starlight's own strings.
 */
import { defineCollection } from 'astro:content';
import { z } from 'astro/zod';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
  i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
  docs: defineCollection({
    loader: docsLoader({
      // Like glob's default id (frontmatter `slug`, else the path without extension or trailing
      // `index`), prefixed with `docs/`: `user-guide/x.md` → `docs/user-guide/x`, `index.md` → `docs`.
      generateId: ({ entry, data }) => {
        const path =
          typeof data.slug === 'string' ? data.slug : entry.replace(/\.\w+$/, '').replace(/(^|\/)index$/, '');
        return ['docs', path].filter(Boolean).join('/');
      },
    }),
    schema: docsSchema({
      extend: z.object({
        // Pull request a prompt-trail entry landed in; shown under the title (src/components/starlight/GitHubLink.astro).
        pr: z.url().optional(),
        // Task file the entry built (`dev/tasks/<NNN>-<slug>.md`); linked under the title, checked by `just dev check`.
        task: z
          .string()
          .regex(/^dev\/tasks\/\d{3}-[a-z0-9-]+\.md$/)
          .optional(),
      }),
    }),
  }),
};
