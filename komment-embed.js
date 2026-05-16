import init, { Komment } from "/pkg/komment.js";

const WORKER_URL = "https://komment.s42.workers.dev";

// Inject styles for the spinner
const style = document.createElement('style');
style.innerHTML = `
    .komment-loading-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 40px;
        color: #57606a;
    }
    .komment-spinner {
        width: 24px;
        height: 24px;
        border: 2px solid rgba(0, 0, 0, 0.1);
        border-top-color: #2da44e;
        border-radius: 50%;
        animation: komment-spin 0.6s linear infinite;
        margin-bottom: 12px;
    }
    .komment-spinner-inline {
        width: 14px;
        height: 14px;
        border: 2px solid rgba(255, 255, 255, 0.3);
        border-top-color: #ffffff;
        border-radius: 50%;
        animation: komment-spin 0.6s linear infinite;
        display: inline-block;
        margin-right: 8px;
        vertical-align: middle;
    }
    .komment-login-container {
        display: flex;
        justify-content: center;
        width: 100%;
        margin-top: 20px;
        padding-bottom: 20px;
    }
    @keyframes komment-spin {
        to { transform: rotate(360deg); }
    }
`;
document.head.appendChild(style);

function getLoadingHtml(text) {
    return `<div class="komment-loading-container">
        <div class="komment-spinner"></div>
        <span>${text}</span>
    </div>`;
}

async function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

window.komment = async function(repo) {
    const container = document.querySelector('.komment');
    if (!container) {
        console.error("Komment: <div class='komment'></div> not found.");
        return;
    }

    container.innerHTML = getLoadingHtml("Loading discussion...");
    
    const loginBtn = document.createElement('button');
    loginBtn.id = 'login-btn';
    loginBtn.innerText = 'Login with GitHub';
    loginBtn.style.display = 'none';

    let instance;
    const term = `komment: ${window.location.origin}${window.location.pathname}`;

    async function handleOAuth() {
        const params = new URLSearchParams(window.location.search);
        const code = params.get("code");
        if (code) {
            try {
                const res = await fetch(`${WORKER_URL}/api/token`, {
                    method: "POST",
                    headers: { "Content-Type": "application/json" },
                    body: JSON.stringify({ code })
                });
                const data = await res.json();
                if (data.access_token) {
                    localStorage.setItem("gh_token", data.access_token);
                }
                window.history.replaceState({}, document.title, window.location.pathname);
            } catch (err) { console.error("Komment Auth Error:", err); }
        }
    }

    async function refresh(retryCount = 0) {
        const token = localStorage.getItem("gh_token");
        try {
            const data = await instance.fetch_discussion();
            instance.render(container.id || 'komment-widget', data);
            
            // If not logged in, show login button at the bottom of comments
            if (!token) {
                const loginDiv = document.createElement('div');
                loginDiv.className = 'komment-login-container';
                loginDiv.appendChild(loginBtn);
                container.appendChild(loginDiv);
                loginBtn.style.display = 'flex';
            }

            attachHandlers();
        } catch (e) {
            if (e === "DISCUSSION_NOT_FOUND" && token) {
                if (retryCount < 3) {
                    container.innerHTML = getLoadingHtml("Waiting for GitHub...");
                    await sleep(2000);
                    return refresh(retryCount + 1);
                } else {
                    await autoCreate();
                }
            } else if (e === "DISCUSSION_NOT_FOUND" || e.toString().includes("401")) {
                container.innerHTML = `<div class="info">
                    <p>Please login with GitHub to view or post comments.</p>
                    <div class="komment-login-container" id="komment-login-placeholder"></div>
                </div>`;
                document.getElementById("komment-login-placeholder").appendChild(loginBtn);
                loginBtn.style.display = 'flex';
            } else {
                container.innerHTML = `<div class="error">Error: ${e}</div>`;
            }
        }
    }

    async function autoCreate() {
        container.innerHTML = getLoadingHtml("Initializing thread...");
        try {
            const [owner, name] = repo.split("/");
            const url = window.location.href;
            const body = `Comments for **${term}**\n\nReference: [${url}](${url})`;
            await instance.create_discussion(owner, name, "General", term, body);
            await sleep(2000);
            await refresh();
        } catch (err) { container.innerHTML = `<div class="error">Failed to create discussion: ${err}</div>`; }
    }

    function attachHandlers() {
        // Post Comment
        const submitBtn = document.getElementById("komment-submit");
        if (submitBtn) {
            submitBtn.onclick = async () => {
                const textarea = document.getElementById("komment-textarea");
                const body = textarea.value.trim();
                if (!body) return;
                submitBtn.disabled = true;
                const originalText = submitBtn.innerHTML;
                submitBtn.innerHTML = `<div class="komment-spinner-inline"></div> Posting...`;
                try {
                    const discId = container.getAttribute("data-discussion-id");
                    await instance.post_comment(discId, body);
                    textarea.value = "";
                    await refresh();
                } catch (err) { alert("Failed to post: " + err); }
                finally { submitBtn.disabled = false; submitBtn.innerHTML = originalText; }
            };
        }

        // Inline Logout
        const logoutBtn = document.getElementById("logout-btn-inline");
        if (logoutBtn) {
            logoutBtn.onclick = () => {
                localStorage.removeItem("gh_token");
                window.location.reload();
            };
        }

        // Delegate Edit/Delete/Save/Cancel
        container.onclick = async (e) => {
            const btn = e.target;
            const id = btn.getAttribute("data-id");
            if (!id) return;

            if (btn.classList.contains("komment-edit-btn")) {
                document.getElementById(`body-${id}`).style.display = "none";
                document.getElementById(`edit-form-${id}`).style.display = "block";
            } else if (btn.classList.contains("komment-cancel-btn")) {
                document.getElementById(`body-${id}`).style.display = "block";
                document.getElementById(`edit-form-${id}`).style.display = "none";
            } else if (btn.classList.contains("komment-save-btn")) {
                const body = document.getElementById(`textarea-${id}`).value.trim();
                if (!body) return;
                btn.disabled = true;
                try { await instance.update_comment(id, body); await refresh(); }
                catch (err) { alert("Error: " + err); btn.disabled = false; }
            } else if (btn.classList.contains("komment-delete-btn")) {
                if (!confirm("Delete this comment?")) return;
                btn.disabled = true;
                try { await instance.delete_comment(id); await refresh(); }
                catch (err) { alert("Error: " + err); btn.disabled = false; }
            }
        };
    }

    // Initialize
    if (!container.id) container.id = 'komment-widget';
    await init();
    await handleOAuth();
    
    const token = localStorage.getItem("gh_token");
    if (!token) {
        loginBtn.style.display = 'flex';
    }

    loginBtn.onclick = () => {
        const CLIENT_ID = "Iv23liQokIChd3ylSI7R";
        const redirectUri = window.location.origin + window.location.pathname;
        window.location.href = `https://github.com/login/oauth/authorize?client_id=${CLIENT_ID}&scope=public_repo&redirect_uri=${encodeURIComponent(redirectUri)}`;
    };

    instance = new Komment({
        repo: repo,
        mapping: "title",
        term: term,
        token: token,
        api_url: `${WORKER_URL}/api/graphql`,
        category: "General"
    });

    await refresh();
};
