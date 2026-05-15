# komment

A proof-of-concept commenting system powered by GitHub Discussions, implemented in **Rust** and compiled to **WebAssembly (WASM)**.

This project aims to replicate the core functionality of [giscus](https://github.com/giscus/giscus) but leverages Rust's performance and safety, running entirely in the browser via WASM.

## Features

- **GitHub GraphQL API Integration**: Fetches discussion data directly from GitHub.
- **WASM-Powered**: High-performance rendering and logic written in Rust.
- **Zero Configuration (almost)**: Minimal setup required to embed in any web page.
- **Customizable**: Easy to extend the Rust logic or CSS styling.

## Prerequisites

To build this project, you need:

- [Rust](https://www.rust-lang.org/tools/install) (2024 edition)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
- A web server to serve the static files (e.g., [miniserve](https://github.com/svenstaro/miniserve), `npx serve`, etc.)

## Getting Started

### 1. Clone the repository

```bash
git clone <repository-url>
cd komment
```

### 2. Build the WASM package

Use `wasm-pack` to compile the Rust code into WebAssembly and generate the JavaScript glue code.

```bash
wasm-pack build --target web
```

This will create a `pkg/` directory in your project root.

### 3. Run the demo

Start a local web server to view the `index.html` file.

```bash
# Using miniserve
miniserve . -p 8000 --index index.html
```

Open your browser and navigate to `http://localhost:8000`.

## Usage

To use `komment` in your own project, follow the pattern in `index.html`:

```html
<div id="komment-container">Loading...</div>

<script type="module">
  import init, { Komment } from "./pkg/komment.js";

  async function run() {
    await init();

    const config = {
      repo: "owner/repo",
      repo_id: "...",
      category: "Announcements",
      category_id: "...",
      mapping: "pathname",
      term: "1", // Discussion number
      token: "YOUR_GITHUB_TOKEN" // Optional for public repos, recommended for higher limits
    };

    const komment = new Komment(config);
    const data = await komment.fetch_discussion();
    komment.render("komment-container", data);
  }

  run();
</script>
```

### Authentication

The GitHub GraphQL API requires authentication for most operations. For development/demo purposes, you can set your token in `localStorage`:

```javascript
localStorage.setItem('github_token', 'your_personal_access_token');
```

In a production environment, you would typically handle this through an OAuth flow, similar to how the original giscus works.

## Documentation

Detailed documentation is available in the `doc/` folder:

- [**HOW-TO-USE.md**](./doc/HOW-TO-USE.md): The main guide for integration and deployment.
- [**DESIGN.md**](./doc/DESIGN.md): Architectural overview and design decisions.
- [**SECURITY.md**](./doc/SECURITY.md): Best practices for securing API tokens.
- [**CLOUDFLARE.md**](./doc/CLOUDFLARE.md): Instructions for the Rust/WASM Cloudflare Worker backend.

## Project Structure

- `src/lib.rs`: The client-side Rust logic for the WASM widget.
- `worker/`: The server-side Rust logic for the Cloudflare Worker.
- `doc/`: Comprehensive project documentation.

## License

Dual-licensed under [MIT](./LICENSE-MIT) and [Apache 2.0](./LICENSE-APACHE).
