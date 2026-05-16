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
- **Homepage URL**: `https://your-domain.com`
- **Callback URL**: `https://your-worker.workers.dev/api/auth/callback`
- **Webhooks**: Uncheck "Active".
- **Permissions**:
  - **Repository permissions** -> **Discussions**: `Read & write`.
- Click **Create GitHub App**.
- Copy the **Client ID**.
- Click **Generate a new client secret** and copy it.

## 3. Configure the Widget
You don't need to modify the `komment-embed.js` source code. Instead, you pass your **Client ID** when you initialize the widget on your page. The `workerUrl` is automatically detected from the script's location, but can be overridden if needed.

```javascript
komment('your-username/your-repo', {
  clientId: 'your-github-client-id' // Required
  // workerUrl: 'https://your-worker.workers.dev', // Optional (auto-detected)
});
```

## 4. Deploy
Deploy the entire stack to Cloudflare.
```bash
# From the project root
just deploy
```

## 5. Set Secrets
Tell your Cloudflare Worker about your GitHub App credentials.
```bash
cd worker
npx wrangler secret put GITHUB_CLIENT_ID
npx wrangler secret put GITHUB_CLIENT_SECRET
```

## 6. Install the App
Crucially, you must install the app on your own repository.
- Go to your GitHub App settings -> **Install App**.
- Click **Install** on your account.
- Select the specific repository you want to use for comments.

## 7. Automatic Setup
Now, simply visit your website while logged in. Komment will detect that no discussion exists for the URL and **automatically create one** in your repository's General category.
