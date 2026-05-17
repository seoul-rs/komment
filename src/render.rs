use wasm_bindgen::prelude::*;
use crate::models::DiscussionResponse;
use crate::config::KommentConfig;

pub fn render_discussion(config: &KommentConfig, element_id: &str, data: JsValue) -> Result<(), JsValue> {
    let response: DiscussionResponse = serde_wasm_bindgen::from_value(data)?;
    
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window found"))?;
    let document = window.document().ok_or_else(|| JsValue::from_str("No document found"))?;
    let container = document.get_element_by_id(element_id).ok_or_else(|| JsValue::from_str("Element not found"))?;

    if let Some(errors) = response.errors {
        if !errors.is_empty() {
            return Err(JsValue::from_str(&format!("GitHub API Error: {}", errors[0].message)));
        }
    }

    let data = response.data.ok_or_else(|| JsValue::from_str("No data in response"))?;
    let viewer_login = data.viewer.as_ref().map(|v| v.login.as_str());

    let discussion = if let Some(repo) = data.repository {
        repo.discussion
    } else if let Some(search) = data.search {
        search.edges.first().and_then(|e| e.node.clone())
    } else {
        None
    };

    let discussion = discussion.ok_or_else(|| JsValue::from_str("DISCUSSION_NOT_FOUND"))?;

    container.set_attribute("data-discussion-id", &discussion.id)?;

    let mut html = format!(
        r#"<div class="komment-discussion">
            <h2>{}</h2>
            <div class="komment-body">{}</div>
            <div class="komment-comments">"#,
        discussion.title, discussion.body_html
    );

    for comment in &discussion.comments.nodes {
        let is_author = viewer_login == Some(&comment.author.login);
        
        html.push_str(&format!(
            r#"<div class="komment-comment" id="comment-{id}">
                <div class="komment-comment-header">
                    <div style="display:flex; align-items:center; gap:10px;">
                        <img src="{avatar}" width="30" height="30" />
                        <strong>{author}</strong> at {date}
                    </div>
                    {actions}
                </div>
                <div class="komment-comment-body" id="body-{id}">{body_html}</div>
                <div class="komment-comment-edit" id="edit-form-{id}" style="display:none; padding:16px;">
                    <textarea id="textarea-{id}" style="width:100%; min-height:80px; margin-bottom:10px;">{body_raw}</textarea>
                    <div style="display:flex; gap:10px;">
                        <button class="komment-save-btn" data-id="{id}">Save</button>
                        <button class="komment-cancel-btn" data-id="{id}">Cancel</button>
                    </div>
                </div>
            </div>"#,
            id = comment.id,
            author = comment.author.login,
            avatar = comment.author.avatar_url,
            date = comment.created_at,
            body_html = comment.body_html,
            body_raw = comment.body.replace('"', "&quot;"),
            actions = if is_author {
                format!(
                    r#"<div class="komment-actions">
                        <button class="komment-edit-btn" data-id="{id}" title="Edit">
                            <svg height="16" viewBox="0 0 16 16" width="16"><path d="M11.013 1.427a.75.75 0 0 1 1.06 0l2.5 2.5a.75.75 0 0 1 0 1.06l-9.5 9.5a.75.75 0 0 1-.53.22H2.25a.75.75 0 0 1-.75-.75v-2.293a.75.75 0 0 1 .22-.53l9.5-9.5Zm.97 1.06L2.97 11.513v1.517h1.517L13.513 4.03l-1.53-1.532Z"></path></svg>
                        </button>
                        <button class="komment-delete-btn" data-id="{id}" title="Delete">
                            <svg height="16" viewBox="0 0 16 16" width="16"><path d="M11 1.75V3h2.25a.75.75 0 0 1 0 1.5H2.75a.75.75 0 0 1 0-1.5H5V1.75C5 .784 5.784 0 6.75 0h2.5C10.216 0 11 .784 11 1.75ZM4.496 6.675a.75.75 0 1 0-1.492.15l.66 6.6A1.75 1.75 0 0 0 5.405 15h5.19c.9 0 1.652-.681 1.741-1.576l.66-6.6a.75.75 0 0 0-1.492-.149l-.66 6.6a.25.25 0 0 1-.249.225h-5.19a.25.25 0 0 1-.249-.225l-.66-6.6Z"></path></svg>
                        </button>
                    </div>"#,
                    id = comment.id
                )
            } else {
                "".to_string()
            }
        ));
    }

    html.push_str("</div>");

    let version = env!("CARGO_PKG_VERSION");
    let branding_html = format!(
        r#"<div style="font-size:11px; color:#57606a; text-align:right; margin-top:10px;">
            powered by <a href="https://github.com/seungjin/komment" target="_blank" style="color:inherit; text-decoration:none; font-weight:600;">Komment</a> v{version}
        </div>"#,
        version = version
    );

    if config.token.is_some() {
        let has_comments = !discussion.comments.nodes.is_empty();
        let editor_style = if has_comments { "" } else { "border-top: none; padding-top: 0; margin-top: 0;" };

        html.push_str(&format!(
            r#"<div class="komment-editor" style="{editor_style}">
                <textarea id="komment-textarea" placeholder="Leave a comment..."></textarea>
                <div style="display:flex; align-items:center; justify-content:space-between; gap:10px;">
                    <div style="display:flex; gap:10px;">
                        <button id="logout-btn-inline" class="komment-logout-btn">
                            <svg height="16" viewBox="0 0 16 16" width="16" style="fill:currentColor;"><path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.31-1.23-.64-1.48 2.05-.23 4.2-.61 4.2-4.13 0-.91-.32-1.65-.84-2.24.08-.21.36-1.07-.08-2.21 0 0-.7-.23-2.29.86-.66-.18-1.37-.27-2.07-.27-.69 0-1.4.09-2.07.27-1.59-1.09-2.29-.86-2.29-.86-.44 1.14-.16 2-.08 2.21-.52.59-.84 1.33-.84 2.24 0 3.51 2.15 3.89 4.2 4.13-.26.21-.51.59-.59.91-.4.18-1.44.49-2.07-.59-.14-.24-.4-.44-.68-.48-.28-.04-.42.01-.1.08.2.06.41.29.54.52.19.34.18.96.48 1.16.2.14.71.1 1.05.07.01.67.01 1.3.01 1.48 0 .21-.15.46-.55.38A8.013 8.013 0 0 1 0 8c0-4.42 3.58-8 8-8z"></path></svg>
                            Logout
                        </button>
                        <button id="komment-submit">Post Comment</button>
                    </div>
                    {branding_html}
                </div>
            </div>"#,
            editor_style = editor_style,
            branding_html = branding_html
        ));
    }

    html.push_str("</div>");


    container.set_inner_html(&html);

    Ok(())
}
