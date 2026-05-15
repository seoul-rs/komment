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
    pub repo_id: String,
    pub category: String,
    pub category_id: String,
    pub mapping: String,
    pub term: String,
    pub token: Option<String>,
    pub api_url: Option<String>,
}


#[derive(Serialize, Deserialize)]
pub struct DiscussionResponse {
    pub data: DiscussionData,
}

#[derive(Serialize, Deserialize)]
pub struct DiscussionData {
    pub repository: Option<RepositoryData>,
    pub search: Option<SearchData>,
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
        let (owner, name) = self.config.repo.split_once('/').ok_or("Invalid repo format")?;
        
        let query = match self.config.mapping.as_str() {
            "number" => format!(
                r#"query {{
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

        let json = JsFuture::from(resp.json()?).await?;
        Ok(json)
    }

    pub fn render(&self, element_id: &str, data: JsValue) -> Result<(), JsValue> {
        let response: DiscussionResponse = serde_wasm_bindgen::from_value(data)?;
        
        let window = web_sys::window().ok_or("No window found")?;
        let document = window.document().ok_or("No document found")?;
        let container = document.get_element_by_id(element_id).ok_or("Element not found")?;

        let discussion = if let Some(repo) = response.data.repository {
            repo.discussion
        } else if let Some(search) = response.data.search {
            search.edges.first().and_then(|e| e.node.clone())
        } else {
            None
        };

        let discussion = discussion.ok_or("Discussion not found")?;

        let mut html = format!(
            r#"<div class="komment-discussion">
                <h2>{}</h2>
                <div class="komment-body">{}</div>
                <div class="komment-comments">"#,
            discussion.title, discussion.body_html
        );

        for comment in discussion.comments.nodes {
            html.push_str(&format!(
                r#"<div class="komment-comment">
                    <div class="komment-comment-header">
                        <img src="{}" width="30" height="30" />
                        <strong>{}</strong> at {}
                    </div>
                    <div class="komment-comment-body">{}</div>
                </div>"#,
                comment.author.avatar_url, comment.author.login, comment.created_at, comment.body_html
            ));
        }

        html.push_str("</div></div>");

        container.set_inner_html(&html);

        Ok(())
    }
}
