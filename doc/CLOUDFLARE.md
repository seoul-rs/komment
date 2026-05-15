# Deploying to Cloudflare Workers

Komment uses the modern Cloudflare Workers + Static Assets architecture, all implemented in Rust.

## Prerequisites

Ensure you have the following installed:
- `worker-build`: `cargo install worker-build`
- `wrangler`: `npm install -g wrangler`

## Unified Configuration

The project is managed through `worker/wrangler.toml`. This file handles:
- **Routing**: Points `main` to the built Rust binary.
- **Assets**: Maps `directory = "../public"` to serve your frontend files.
- **Build**: Defines the custom command `worker-build --release` to compile the Rust worker logic.

## Deployment Steps

1. **Rebuild the Frontend WASM**:
   ```bash
   wasm-pack build --target web
   ```

2. **Sync Public Assets**:
   Ensure `public/` contains your `index.html` and `pkg/` folder.

3. **Deploy with Wrangler**:
   ```bash
   cd worker
   wrangler deploy
   ```

Alternatively, use the provided **Justfile** to automate all three steps:
```bash
just deploy
```

## Environment Variables (Secrets)

For security, GitHub credentials are not stored in the configuration file. You must set them as secrets:

```bash
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
```

## Troubleshooting

- **404 Errors**: Ensure your `wrangler.toml` has the correct assets directory path.
- **500 Errors**: Check your worker logs with `wrangler tail`. Most 500 errors in the worker are related to missing secrets or incorrect GitHub App permissions.
- **WASM Load Failures**: Ensure your frontend imports use absolute paths (`/pkg/komment.js`) to support sub-directories.
