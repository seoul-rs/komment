# komment

A high-performance commenting system powered by **Rust**, **WebAssembly (WASM)**, and **GitHub Discussions**.

Komment provides a secure, fast, and modern way to add discussions to your website without managing a database. It mirrors the core functionality of [giscus](https://giscus.app) but is built entirely with Rust on both the client and the server.

## Features

- **Blazing Fast**: Powered by Rust compiled to WASM.
- **No Database**: Uses GitHub Discussions as the data store.
- **Zero-Config Threads**: Automatically creates discussion threads for every unique URL on your site.
- **Full CRUD**: Post, Edit, and Delete comments directly from your site.
- **Unified Architecture**: Both frontend assets and the OAuth relay run on a single Cloudflare Worker.
- **Secure**: Uses a Cloudflare Worker proxy to safely exchange OAuth codes for tokens.

## Quick Start (Embedded)

To use `komment` on any website, simply add the following code:

```html
<!-- 1. The container -->
<div class="komment"></div>

<!-- 2. The script (Change URL to your deployed worker) -->
<script src="https://komment.s42.workers.dev/komment-embed.js" type="module"></script>

<!-- 3. Initialize -->
<script type="module">
  import "/komment-embed.js";
  komment('your-username/your-repo');
</script>
```

## Setup & Deployment

### Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- [worker-build](https://github.com/cloudflare/workers-rs) (`cargo install worker-build`)
- [Cloudflare Wrangler CLI](https://developers.cloudflare.com/workers/wrangler/install-and-update/)

### 1. Build and Deploy
The project uses a `justfile` for easy management.

```bash
# Rebuild WASM and deploy everything to Cloudflare
just deploy
```

### 2. Configure GitHub App
1. Create a [GitHub App](https://github.com/settings/apps/new).
2. Set **Callback URL** to your worker domain (e.g., `https://komment.your-name.workers.dev/`).
3. Under **Permissions**, set **Discussions** to `Read & write`.
4. Enable **Discussions** in your target repository settings.

### 3. Set Secrets
Run these in the `worker/` directory:
```bash
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
```

## Documentation

- [**HOW-TO-USE.md**](./doc/HOW-TO-USE.md): Step-by-step setup guide.
- [**DESIGN.md**](./doc/DESIGN.md): Internal architecture and design decisions.
- [**CLOUDFLARE.md**](./doc/CLOUDFLARE.md): Worker-specific deployment details.

## License

Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE).
Copyright (c) 2026 Seungjin Kim.
