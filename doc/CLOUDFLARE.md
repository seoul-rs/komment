# Deploying to Cloudflare Workers (The Komment Architecture)

This guide explains how to use a Cloudflare Worker as a secure, high-performance relay for your GitHub App. This architecture perfectly mirrors how the original `komment` works.

## 1. How it works
Your Cloudflare Worker acts as a **Relay**:
- It keeps your GitHub App's `Client Secret` hidden.
- It handles the authentication handshake (`/api/token`).
- It proxies all GraphQL requests (`/api/graphql`) to GitHub, ensuring your API interactions are centralized and can be secured with CORS.

## 2. Setup
...
3. **Deploy**:
   ```bash
   wrangler deploy
   ```

## 3. Integration Code Example

```javascript
import init, { Komment } from "./pkg/komment.js";

// Your Cloudflare Worker base URL
const WORKER_BASE = "https://komment-wasm-worker.s42.workers.dev";

async function run() {
  await init();

  // 1. Handle Login Callback
  const code = new URLSearchParams(window.location.search).get("code");
  if (code) {
    const res = await fetch(`${WORKER_BASE}/api/token`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code })
    });
    const result = await res.json();
    localStorage.setItem('gh_token', result.access_token);
  }

  // 2. Setup Widget
  const config = {
    repo: "owner/repo",
    mapping: "title",
    term: window.location.href,
    token: localStorage.getItem('gh_token'),
    api_url: `${WORKER_BASE}/api/graphql` // Proxy all calls through your worker!
  };

  const komment = new Komment(config);
  const data = await komment.fetch_discussion();
  komment.render("komment", data);
}
```

## 4. Advantages of Cloudflare Workers + Rust
- **Latency**: Workers run on Cloudflare's edge, meaning extremely fast response times globally.
- **Cold Starts**: WASM-based workers have near-zero cold start times compared to Node.js.
- **Type Safety**: Using Rust on both the client and server ensures a robust and predictable system.
- **Cost**: The free tier covers 100,000 requests per day.
