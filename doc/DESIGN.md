# Design & Architecture: Komment

Komment is built for speed, security, and low maintenance. It leverages the latest features of Rust, WebAssembly, and Cloudflare Workers.

## High-Level Architecture

```text
[ Browser ] <---> [ Cloudflare Worker ] <---> [ GitHub API ]
     |                  |
   (WASM)          (Rust Logic)
     |                  |
[ DOM UI ]         [ OAuth Proxy ]
```

### 1. Client-Side (Rust + WASM)
The core widget is a Rust library compiled to WebAssembly.
- **`wasm-bindgen`**: Bridges the gap between Rust and JavaScript.
- **`web-sys`**: Direct DOM manipulation from Rust for rendering comments and forms.
- **Logic**: Handles mapping the current URL to a GitHub Discussion thread title and managing the CRUD state of comments.

### 2. Server-Side (Rust + Cloudflare Worker)
A single Rust-based Cloudflare Worker performs two critical roles:
- **Assets Host**: Serves the `index.html`, `komment-embed.js`, and the WASM binary.
- **OAuth Proxy**: Safely handles the GitHub App Client Secret to exchange codes for access tokens. It also provides a centralized callback endpoint (`/api/auth/callback`) to support multiple domains via the `state` parameter redirect.
- **GraphQL Proxy**: Forwards authenticated requests to GitHub, ensuring a single consistent origin for the widget.

### 3. Data Store (GitHub Discussions)
No database is required.
- **Search**: Threads are discovered by searching for a specific discussion title (format: `komment: host.com/path`).
- **Auto-Provisioning**: If no matching thread is found, the system performs a `createDiscussion` mutation instantly, including a reference link back to the source page.

## Key Design Decisions

- **Zero-Config Styling**: All CSS is bundled and injected by the embed script, allowing the widget to be used on any site with a single line of HTML and no external CSS files.
- **Portable Dependencies**: Uses `import.meta.url` to resolve WASM dependencies and the `workerUrl` relative to the script's hosting location, enabling CDN usage and zero-config deployment.
- **Eventual Consistency**: Includes a retry mechanism in the frontend to handle the short delay between discussion creation and API search availability.
