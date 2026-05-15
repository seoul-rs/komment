# Design Document: komment-wasm

## Overview
`komment-wasm` is a Rust-based implementation of a GitHub Discussions commenting system, compiled to WebAssembly. It serves as a high-performance, type-safe alternative to traditional JavaScript-heavy widgets.

## Architecture

### 1. The Rust-WASM Bridge
We use `wasm-bindgen` to create a seamless interface between the browser's JavaScript environment and the Rust logic.
- **`Komment` Struct**: The main state container, holding configuration like repository details and authentication tokens.
- **Type Mapping**: Serde is used to bridge complex JSON responses from GitHub's GraphQL API into strongly-typed Rust structures.

### 2. Networking
Instead of using a bulky Rust HTTP client, we leverage `web-sys` to access the browser's native `fetch` API. This keeps the WASM binary small and utilizes the browser's built-in connection pooling and security features (like CORS handling).

### 3. Rendering Strategy
The current implementation uses a **Direct DOM Manipulation** strategy:
- Rust logic constructs HTML strings based on the retrieved data.
- It uses `web_sys::Document` and `web_sys::Element` to inject this HTML into the target container.
- *Future Consideration*: Moving to a Virtual DOM approach (like `yew` or `leptos`) if the UI complexity grows significantly.

### 4. Discussion Mapping Strategy
The library supports two primary methods for connecting a web page to a GitHub Discussion:
- **Direct Mapping (`number`)**: Uses the specific discussion number. This is the most efficient as it uses a direct repository query.
- **Search-based Mapping (`title`, `pathname`, etc.)**: Uses the GitHub GraphQL `search` API to find a discussion whose title matches a specific term (like the page URL). This is more flexible for dynamic websites.

### 5. Data Flow
1. **Initialization**: JS calls `init()` and instantiates `new Komment(config)`.
2. **Fetching**: `fetch_discussion()` triggers an asynchronous GraphQL request via the browser's `fetch`.
3. **Processing**: Rust deserializes the JSON, handles potential missing fields, and prepares the data for rendering.
4. **Rendering**: `render()` takes the processed data and updates the DOM.

## Security
- **Token Handling**: Tokens are passed from JS to Rust. The implementation ensures they are only used in the `Authorization` header of the fetch request.
- **Sanitization**: Currently relies on browser-side HTML rendering. Future versions should include a Rust-based HTML sanitizer for comment bodies if we move away from `set_inner_html`.

## Performance Goals
- **Minimal Payload**: Aiming for a small `.wasm` file size by limiting dependencies.
- **Fast Execution**: Utilizing Rust's speed for data processing and HTML string construction.
