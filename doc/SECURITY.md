# Security Guide: Protecting Secrets in komment-wasm

This document explains how to handle sensitive information like GitHub API tokens when using this library.

## The Challenge
Since `komment-wasm` runs in the user's browser, any configuration value (like a token) passed to it is visible. If you use a Personal Access Token (PAT) with write access, anyone visiting your site could potentially steal it and use it to modify your repository.

## Solution: The Backend Proxy
The only secure way to use a sensitive token is to never send it to the browser. Instead, you use a proxy server.

### How it Works
1. **WASM Client**: Sends a request to your server (e.g., `/api/github-proxy`).
2. **Your Server**: 
   - Receives the request.
   - Attaches the `Authorization: Bearer <GITHUB_TOKEN>` header.
   - Forwards the request to GitHub's GraphQL API.
   - Returns the response to the WASM client.
3. **GitHub**: Sees the request coming from your server, which is an authorized source.

### Setup Instructions

#### 1. Configure your Server
Use the **Cloudflare Worker (Rust/WASM)** implementation in the `worker/` directory.
- **Backend**: Rust-based worker running on the edge.
- **Environment**: Set `GITHUB_CLIENT_ID` and `GITHUB_CLIENT_SECRET` via Wrangler secrets.
- **Access Control**: Configure origins in `worker/src/lib.rs`.

#### 2. Configure komment-wasm
Set the `TOKEN_ENDPOINT` in your frontend integration to point to your Cloudflare Worker:

```javascript
const config = {
  repo: "owner/repo",
  mapping: "title",
  term: window.location.href,
  api_url: "https://your-site.com/api/github-proxy"
  // DO NOT include the token here
};
```

## Alternative: Read-Only Tokens
If your integration only **reads** public discussions and doesn't allow posting comments, you can use a **Fine-grained Personal Access Token** with:
- **Scope**: `Read-only` for `Discussions`.
- **Repository Access**: Only the specific repository you are using.

While this token is still visible in the browser, its potential for abuse is significantly lower than a general-purpose token.

## Summary of Best Practices
1. **Never** use a token with write access in the browser.
2. **Always** use a proxy for production deployments.
3. **Restrict** CORS on your proxy to your own domain.
4. **Rate limit** your proxy to prevent API quota exhaustion.
