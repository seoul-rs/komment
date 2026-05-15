# Definitive Guide: Using komment-wasm

This guide provides a step-by-step roadmap to setting up your own secure, high-performance commenting system using Rust, WASM, and Cloudflare Workers.

## Workflow Overview
1. **Build**: Compile the Rust code to WebAssembly.
2. **Authorize**: Create a GitHub App to act as the identity provider.
3. **Secure**: Deploy a Cloudflare Worker to handle secrets.
4. **Integrate**: Embed the widget into your site.

---

## Step 1: Build the WASM Package
First, compile your Rust logic into a package that the browser can understand.

```bash
wasm-pack build --target web
```
This generates the `pkg/` directory containing `komment.js` and `komment_bg.wasm`.

---

## Step 2: Setup the GitHub App (The Identity Provider)
To allow users to comment, you need a GitHub App.

1. Go to **Settings > Developer settings > GitHub Apps > New GitHub App**.
2. **Permissions**: Set `Discussions` to `Read & Write`.
3. **User Permissions**: Set `Email addresses` to `Read-only`.
4. **Generate Secret**: Save the **Client ID** and **Client Secret**.

---

## Step 3: Deploy the Backend (The Gatekeeper)
You need a secure place to store your `Client Secret`. We use a **Cloudflare Worker** (Rust/WASM) for this to maintain a full Rust ecosystem.

See [**doc/CLOUDFLARE.md**](./CLOUDFLARE.md) for detailed setup and deployment instructions.

---

## Step 4: Embed in Your Website
Add this code to your website where you want the comments to appear.

### HTML Structure
```html
<div id="komment">Loading comments...</div>
<button id="login-btn" style="display:none">Login with GitHub to comment</button>
```

### Integration Logic
```javascript
import init, { Komment } from "./pkg/komment.js";

// Your Cloudflare Worker endpoint
const TOKEN_ENDPOINT = "https://your-worker.your-username.workers.dev/api/token"; 

async function startKomment() {
  await init();

  // 1. Handle OAuth Callback
  const params = new URLSearchParams(window.location.search);
  const code = params.get("code");
  if (code) {
    const res = await fetch(TOKEN_ENDPOINT, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ code })
    });
    const { access_token } = await res.json();
    localStorage.setItem("gh_token", access_token);
    window.history.replaceState({}, document.title, window.location.pathname);
  }

  // 2. Setup Configuration
  const token = localStorage.getItem("gh_token");
  const config = {
    repo: "your-owner/your-repo",
    mapping: "title",
    term: window.location.href, // Maps comments to this URL
    token: token
  };

  // 3. Render
  const komment = new Komment(config);
  try {
    const data = await komment.fetch_discussion();
    komment.render("komment", data);
  } catch (e) {
    console.log("No discussion found or not logged in.");
    if (!token) document.getElementById("login-btn").style.display = "block";
  }
}

// 4. Handle Login Button
document.getElementById("login-btn").onclick = () => {
  const client_id = "YOUR_GITHUB_CLIENT_ID";
  window.location.href = `https://github.com/login/oauth/authorize?client_id=${client_id}&scope=public_repo`;
};

startKomment();
```

---

## Configuration Reference

| Option | Description |
| :--- | :--- |
| `repo` | Your repository in `owner/name` format. |
| `mapping` | Set to `"title"` to use the page URL as the discussion title. |
| `term` | The unique identifier (usually `window.location.href`). |
| `token` | The access token retrieved from your Cloudflare Worker. |

## Why this is secure
- **The Secret** is stored in Cloudflare's environment/secrets, never in the browser.
- **The User Token** is stored in the user's `localStorage` and only grants access to perform actions as that specific user.
- **WASM** provides a fast, type-safe layer for processing the GitHub API data before it hits your DOM.
