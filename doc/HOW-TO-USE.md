# Definitive Guide: Using Komment

Follow these steps to set up your own secure, high-performance commenting system.

## 1. Prepare your GitHub Repository
Komment stores all data in GitHub Discussions.
- Go to your repository **Settings**.
- Scroll to **Features** and check **Discussions**.
- Ensure you have a category named **General** (this is the default).

## 2. Create a GitHub App
You need a GitHub App to handle user logins securely.
- Go to **[GitHub App Settings](https://github.com/settings/apps/new)**.
- **GitHub App name**: `Komment-YourName`
- **Homepage URL**: `https://your-domain.com` (The site where you'll host the comments)
- **Callback URL**: `https://your-worker.workers.dev/api/auth/callback` (Crucial: must end in `/api/auth/callback`)
- **Webhooks**: Uncheck "Active".
- **Permissions**:
  - **Repository permissions** -> **Discussions**: `Read & write`.
- Click **Create GitHub App**.
- Copy the **Client ID**.
- Click **Generate a new client secret** and copy it.

## 3. Deploy to Cloudflare
Komment is designed to run on a Cloudflare Worker with static assets.
```bash
# 1. From the project root, build and deploy
just deploy

# 2. Set your GitHub App secrets
cd worker
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
```

## 4. Install the App
Crucially, you must install the app on the repository where you want comments to appear.
- Go to your GitHub App settings -> **Install App**.
- Click **Install** on your account.
- Select the specific repository (e.g., `your-username/your-repo`).

## 5. Embed on Your Site
You can now embed the widget on any page. You don't need to host the script on the same server; you can load it directly from your Cloudflare Worker or any CDN where you've copied the `public/` files.

```html
<div class="komment"></div>

<script type="module">
  // 1. Load the script (from your worker or CDN)
  import "https://your-worker.workers.dev/komment-embed.js";
  
  // 2. Initialize
  komment('your-username/your-repo', {
    clientId: 'your-github-client-id' // Required
  });
</script>
```

## 6. Automatic Setup
Simply visit your website while logged in with GitHub. Komment will:
1. Detect that no discussion exists for the page.
2. **Automatically create one** in your repository.
3. Add a reference link in the discussion body back to your page.

---

### Pro-Tips & Troubleshooting
- **CORS**: The widget automatically includes a `_headers` file to allow cross-origin script loading.
- **Worker URL**: If your script is on a different domain than your worker, the widget automatically detects the correct worker URL from the script's source.
- **Icons**: Action buttons (Edit/Delete) use SVG icons and will automatically adapt to your site's theme.
