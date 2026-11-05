# Stackific OS website

Built with [Astro](https://docs.astro.build) and [Starlight](https://starlight.astro.build), run with Bun.

- Marketing pages: `src/pages/` → served at `/`.
- Docs: `src/content/docs/` (`.md`/`.mdx`; each file becomes a route) → served at `/docs/`. Sidebar is in `astro.config.mjs`.
- Images go in `src/assets/`, static files in `public/`.
- Typography: shared fonts in `src/styles/fonts.css`; Starlight overrides in `src/styles/starlight.css`.

## Commands

Run from the repo root:

| Command             | Action                                  |
| :------------------ | :-------------------------------------- |
| `just setup`        | Install Bun and site dependencies       |
| `just site dev`     | Dev server at `localhost:4321`          |
| `just site build`   | Build the site into `site/dist/`        |
| `just site preview` | Build, then preview the production site |

Recipes are defined in `site/justfile`.

## Deploy (Cloudflare Pages)

The site is fully static (`dist/`), so Pages serves it as-is: no Functions, adapter or server code.

Deploys go through Pages' Git integration (builds on every push); nothing Cloudflare-specific is installed locally. Create a Pages project from the repo with:

| Setting                | Value                                                                                                             |
| :--------------------- | :---------------------------------------------------------------------------------------------------------------- |
| Framework preset       | None                                                                                                              |
| Root directory         | `site`                                                                                                            |
| Build command          | `bun install --frozen-lockfile && bun --bun run build`                                                            |
| Build output directory | `dist`                                                                                                            |
| Environment variables  | `BUN_VERSION=1.4.4` (match `just/bun/vars.just`), `SKIP_DEPENDENCY_INSTALL=1` (stops Pages running `npm install`) |

The production URL is set in `astro.config.mjs` (`site`); keep it in sync with the Pages custom domain, as the sitemap uses it.
