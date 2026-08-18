# Deploying lawsynth.dev (Cloudflare Pages)

The documentation site is a **deterministic static tree** — every page is a
self-contained HTML document (inline styles + theme script), so it deploys to
Cloudflare Pages with no Workers/Functions required.

## Build the site

From `apps/docs-site/`:

```bash
npm ci            # first time, from the repo root workspace
npm run render    # tsc build + emit the static tree to ./public
```

`npm run render` writes:

- `public/index.html` and `public/<page>/index.html` — one per site page
- `public/sitemap.xml` — the crawl sitemap (origin `https://lawsynth.dev`)
- `public/robots.txt` — allow-all + sitemap pointer
- `public/_headers` — security headers (nosniff, DENY framing, a strict CSP)
- `public/_redirects` — Cloudflare Pages redirect rules (add canonical rules here)

The output is byte-identical across runs (a test asserts this), so a rebuild only
changes what the content changed.

## Deploy

### Option A — direct upload (fastest)

```bash
npm run deploy
# = npm run render && wrangler pages deploy public --project-name lawsynth
```

(Requires `wrangler` and a Cloudflare login: `npx wrangler login`.)

### Option B — Git-connected Pages project

In the Cloudflare dashboard → Pages → *Create project* → connect the repo, then:

| Setting            | Value                    |
| ------------------ | ------------------------ |
| Root directory     | `apps/docs-site`         |
| Build command      | `npm run render`         |
| Build output dir   | `public`                 |

Cloudflare auto-provisions the certificate; `.dev` enforces HTTPS at the TLD
(HSTS preload), so the site is HTTPS-only by construction.

## Custom domain

Point the `lawsynth.dev` apex (and `www`) at the Pages project under
*Custom domains*. DNS lives in the same Cloudflare account as the registration,
so this is a one-click attach.

## What is *not* here yet

A live in-browser discovery **playground** would run the engine via the
`lawsynth-wasm` crate inside a Cloudflare **Worker** (or client-side WASM). That
crate is WASM-compilable but still binding-agnostic (no `wasm-bindgen` layer),
so the playground is a tracked follow-up, not part of this static deploy.
