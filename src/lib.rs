use wasm_bindgen::prelude::*;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};
use wasm_bindgen_futures::JsFuture;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[derive(Serialize, Deserialize, Clone)]
pub struct KommentConfig {
    pub repo: String,
    pub repo_id: Option<String>,
    pub category: Option<String>,
    pub category_id: Option<String>,
    pub mapping: String,
    pub term: String,
    pub token: Option<String>,
    pub api_url: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DiscussionResponse {
    pub data: Option<DiscussionData>,
    pub errors: Option<Vec<GithubError>>,
}

#[derive(Serialize, Deserialize)]
pub struct GithubError {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct DiscussionData {
    pub repository: Option<RepositoryData>,
    pub search: Option<SearchData>,
    pub viewer: Option<ViewerData>,
}

#[derive(Serialize, Deserialize)]
pub struct ViewerData {
    pub login: String,
}

#[derive(Serialize, Deserialize)]
pub struct SearchData {
    pub edges: Vec<SearchEdge>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchEdge {
    pub node: Option<Discussion>,
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryData {
    pub discussion: Option<Discussion>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Discussion {
    pub id: String,
    pub title: String,
    #[serde(rename = "bodyHTML")]
    pub body_html: String,
    pub comments: CommentsConnection,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentsConnection {
    pub nodes: Vec<Comment>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Comment {
    pub id: String,
    pub author: Author,
    #[serde(rename = "bodyHTML")]
    pub body_html: String,
    #[serde(rename = "body")]
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Author {
    pub login: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: String,
}

#[wasm_bindgen]
pub struct Komment {
    config: KommentConfig,
}

#[wasm_bindgen]
impl Komment {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Result<Komment, JsValue> {
        let config: KommentConfig = serde_wasm_bindgen::from_value(config)?;
        Ok(Komment { config })
    }

    pub async fn fetch_discussion(&self) -> Result<JsValue, JsValue> {
        if self.config.token.is_none() {
            return Err(JsValue::from_str("401 Unauthorized: Please login to view comments"));
        }

        let (owner, name) = self.config.repo.split_once('/').ok_or("Invalid repo format")?;
        
        let query = match self.config.mapping.as_str() {
            "number" => format!(
                r#"query {{
                    viewer {{ login }}
                    repository(owner: "{owner}", name: "{name}") {{
                        discussion(number: {number}) {{
                            id
                            title
                            bodyHTML
                            comments(first: 100) {{
                                nodes {{
                                    id
                                    author {{
                                        login
                                        avatarUrl
                                    }}
                                    bodyHTML
                                    body
                                    createdAt
                                }}
                            }}
                        }}
                    }}
                }}"#,
                owner = owner,
                name = name,
                number = self.config.term
            ),
            _ => format!(
                r#"query {{
                    viewer {{ login }}
                    search(query: "repo:{owner}/{name} is:discussion \"{term}\"", type: DISCUSSION, first: 1) {{
                        edges {{
                            node {{
                                ... on Discussion {{
                                    id
                                    title
                                    bodyHTML
                                    comments(first: 100) {{
                                        nodes {{
                                            id
                                            author {{
                                                login
                                                avatarUrl
                                            }}
                                            bodyHTML
                                            body
                                            createdAt
                                        }}
                                    }}
                                }}
                            }}
                        }}
                    }}
                }}"#,
                owner = owner,
                name = name,
                term = self.config.term
            ),
        };

        self.execute_graphql(query).await
    }

    pub async fn create_discussion(&self, repo_owner: String, repo_name: String, category_name: String, title: String, body: String) -> Result<JsValue, JsValue> {
        let query_ids = format!(
            r#"query {{
                repository(owner: "{repo_owner}", name: "{repo_name}") {{
                    id
                    discussionCategories(first: 10) {{
                        nodes {{
                            id
                            name
                        }}
                    }}
                }}
            }}"#,
            repo_owner = repo_owner,
            repo_name = repo_name
        );

        let ids_resp = self.execute_graphql(query_ids).await?;
        let ids_data: serde_json::Value = serde_wasm_bindgen::from_value(ids_resp)?;
        
        let repo_id = ids_data["data"]["repository"]["id"].as_str()
            .ok_or("Could not find repository ID")?;
            
        let category_id = ids_data["data"]["repository"]["discussionCategories"]["nodes"]
            .as_array()
            .ok_or("Could not find categories")?
            .iter()
            .find(|c| c["name"].as_str().map(|n| n.to_lowercase()) == Some(category_name.to_lowercase()))
            .and_then(|c| c["id"].as_str())
            .ok_or_else(|| format!("Category '{}' not found", category_name))?;

        let mutation = format!(
            r#"mutation {{
                createDiscussion(input: {{repositoryId: "{repo_id}", categoryId: "{category_id}", title: {title}, body: {body}}}) {{
                    discussion {{
                        id
                    }}
                }}
            }}"#,
            repo_id = repo_id,
            category_id = category_id,
            title = serde_json::to_string(&title).unwrap_or_else(|_| format!("\"{}\"", title.replace('"', "\\\""))),
            body = serde_json::to_string(&body).unwrap_or_else(|_| format!("\"{}\"", body.replace('"', "\\\"")))
        );

        self.execute_graphql(mutation).await
    }

    async fn execute_graphql(&self, query: String) -> Result<JsValue, JsValue> {
        let opts = RequestInit::new();
        opts.set_method("POST");
        opts.set_mode(RequestMode::Cors);

        let headers = Headers::new()?;
        if let Some(token) = &self.config.token {
            headers.append("Authorization", &format!("Bearer {}", token))?;
        }
        opts.set_headers(&headers);

        let body = serde_json::json!({ "query": query });
        opts.set_body(&JsValue::from_str(&body.to_string()));

        let url = self.config.api_url.as_deref().unwrap_or("https://api.github.com/graphql");
        let request = Request::new_with_str_and_init(url, &opts)?;

        let window = web_sys::window().ok_or("No window found")?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP error: {}", resp.status())));
        }

        let json_value = JsFuture::from(resp.json()?).await?;
        let json_serde: serde_json::Value = serde_wasm_bindgen::from_value(json_value.clone())?;

        // Check for GraphQL errors
        if let Some(errors) = json_serde["errors"].as_array() {
            if !errors.is_empty() {
                let msg = errors[0]["message"].as_str().unwrap_or("Unknown GraphQL error");
                return Err(JsValue::from_str(&format!("GitHub API Error: {}", msg)));
            }
        }

        Ok(json_value)
    }

    pub async fn post_comment(&self, discussion_id: String, body: String) -> Result<JsValue, JsValue> {
        let query = format!(
            r#"mutation {{
                addDiscussionComment(input: {{discussionId: "{discussion_id}", body: {body}}}) {{
                    comment {{
                        id
                    }}
                }}
            }}"#,
            discussion_id = discussion_id,
            body = serde_json::to_string(&body).unwrap_or_else(|_| format!("\"{}\"", body.replace('"', "\\\"").replace('\n', "\\n")))
        );

        self.execute_graphql(query).await
    }

    pub async fn delete_comment(&self, comment_id: String) -> Result<JsValue, JsValue> {
        let query = format!(
            r#"mutation {{
                deleteDiscussionComment(input: {{id: "{comment_id}"}}) {{
                    comment {{
                        id
                    }}
                }}
            }}"#,
            comment_id = comment_id
        );

        self.execute_graphql(query).await
    }

    pub async fn update_comment(&self, comment_id: String, body: String) -> Result<JsValue, JsValue> {
        let query = format!(
            r#"mutation {{
                updateDiscussionComment(input: {{commentId: "{comment_id}", body: {body}}}) {{
                    comment {{
                        id
                    }}
                }}
            }}"#,
            comment_id = comment_id,
            body = serde_json::to_string(&body).unwrap_or_else(|_| format!("\"{}\"", body.replace('"', "\\\"").replace('\n', "\\n")))
        );

        self.execute_graphql(query).await
    }

    pub fn render(&self, element_id: &str, data: JsValue) -> Result<(), JsValue> {
        let response: DiscussionResponse = serde_wasm_bindgen::from_value(data)?;
        
        let window = web_sys::window().ok_or("No window found")?;
        let document = window.document().ok_or("No document found")?;
        let container = document.get_element_by_id(element_id).ok_or("Element not found")?;

        // Note: errors are now caught in execute_graphql, but we keep this for safety
        if let Some(errors) = response.errors {
            if !errors.is_empty() {
                return Err(JsValue::from_str(&format!("GitHub API Error: {}", errors[0].message)));
            }
        }

        let data = response.data.ok_or("No data in response")?;
        let viewer_login = data.viewer.as_ref().map(|v| v.login.as_str());

        let discussion = if let Some(repo) = data.repository {
            repo.discussion
        } else if let Some(search) = data.search {
            search.edges.first().and_then(|e| e.node.clone())
        } else {
            None
        };

        let discussion = discussion.ok_or("DISCUSSION_NOT_FOUND")?;

        container.set_attribute("data-discussion-id", &discussion.id)?;

        let mut html = format!(
            r#"<div class="komment-discussion">
                <h2>{}</h2>
                <div class="komment-body">{}</div>
                <div class="komment-comments">"#,
            discussion.title, discussion.body_html
        );

        for comment in discussion.comments.nodes {
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

        if self.config.token.is_some() {
            html.push_str(r#"
                <div class="komment-editor">
                    <textarea id="komment-textarea" placeholder="Leave a comment..."></textarea>
                    <div style="display:flex; gap:10px;">
                        <button id="logout-btn-inline" class="komment-logout-btn">
                            <svg height="16" viewBox="0 0 16 16" width="16" style="fill:currentColor;"><path d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.31-1.23-.64-1.48 2.05-.23 4.2-.61 4.2-4.13 0-.91-.32-1.65-.84-2.24.08-.21.36-1.07-.08-2.21 0 0-.7-.23-2.29.86-.66-.18-1.37-.27-2.07-.27-.69 0-1.4.09-2.07.27-1.59-1.09-2.29-.86-2.29-.86-.44 1.14-.16 2-.08 2.21-.52.59-.84 1.33-.84 2.24 0 3.51 2.15 3.89 4.2 4.13-.26.21-.51.59-.59.91-.4.18-1.44.49-2.07-.59-.14-.24-.4-.44-.68-.48-.28-.04-.42.01-.1.08.2.06.41.29.54.52.19.34.18.96.48 1.16.2.14.71.1 1.05.07.01.67.01 1.3.01 1.48 0 .21-.15.46-.55.38A8.013 8.013 0 0 1 0 8c0-4.42 3.58-8 8-8z"></path></svg>
                            Logout
                        </button>
                        <button id="komment-submit">Post Comment</button>
                    </div>
                </div>
            "#);
        }

        html.push_str("</div>");

        container.set_inner_html(&html);

        Ok(())
    }
}
