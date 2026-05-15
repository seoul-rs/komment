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
- **OAuth Proxy**: Safely handles the GitHub App Client Secret to exchange codes for access tokens, preventing secrets from leaking to the frontend.
- **GraphQL Proxy**: Forwards authenticated requests to GitHub, ensuring a single consistent origin for the widget.

### 3. Data Store (GitHub Discussions)
No database is required.
- **Search**: Threads are discovered by searching for a specific discussion title (default: the page URL).
- **Auto-Provisioning**: If no matching thread is found, the system performs a `createDiscussion` mutation instantly.
- **Social**: Inherits all the power of GitHub, including reactions, threading, and spam protection.

## Key Design Decisions

- **Eventual Consistency**: Includes a retry mechanism in the frontend to handle the short delay between discussion creation and API search availability.
- **Unified Domain**: Using Cloudflare's `[assets]` configuration, the frontend and API share the same origin, eliminating CORS complexity.
- **One-Line Embed**: The `komment-embed.js` wrapper abstracts the complexities of WASM initialization and authentication flow for the end-user.
