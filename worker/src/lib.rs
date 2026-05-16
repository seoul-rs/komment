use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Deserialize)]
struct TokenRequest {
    code: String,
}

#[derive(Serialize)]
struct GithubTokenRequest {
    client_id: String,
    client_secret: String,
    code: String,
}

#[derive(Deserialize, Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<serde_json::Value>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: worker::Context) -> Result<Response> {
    // 1. Handle CORS Preflight
    let cors = Cors::default()
        .with_origins(vec!["*"])
        .with_methods(vec![Method::Post, Method::Options])
        .with_allowed_headers(vec!["Content-Type", "Authorization"]);

    if req.method() == Method::Options {
        return Response::empty()?.with_cors(&cors);
    }

    let router = Router::new();

    router
        .get("/api/auth/callback", |req, _ctx| {
            let url = req.url()?;
            let query: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
            let code = query.get("code").ok_or("Missing code")?;
            let state = query.get("state").ok_or("Missing state")?;

            // Redirect back to the original site with the code
            let mut redirect_url = Url::parse(state).map_err(|e| worker::Error::from(e.to_string()))?;
            redirect_url.query_pairs_mut().append_pair("code", code);

            Response::redirect(redirect_url)
        })
        .post_async("/api/token", |mut req, ctx| async move {
            let data: TokenRequest = req.json().await?;
            let client_id = ctx.env.var("GITHUB_CLIENT_ID")?.to_string();
            let client_secret = ctx.env.var("GITHUB_CLIENT_SECRET")?.to_string();

            let client = reqwest::Client::new();
            let res = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .json(&GithubTokenRequest {
                    client_id,
                    client_secret,
                    code: data.code,
                })
                .send()
                .await
                .map_err(|e| worker::Error::from(e.to_string()))?;

            let body = res.text().await.map_err(|e| worker::Error::from(e.to_string()))?;
            Response::ok(body)
        })
        .post_async("/api/graphql", |mut req, _ctx| async move {
            let token = match req.headers().get("Authorization")? {
                Some(t) => t,
                None => return Response::error("Unauthorized: Missing Authorization header", 401),
            };
            let graphql_body: GraphQLRequest = req.json().await?;

            let client = reqwest::Client::new();
            let res = client
                .post("https://api.github.com/graphql")
                .header("User-Agent", "komment-worker")
                .header("Authorization", token)
                .json(&graphql_body)
                .send()
                .await
                .map_err(|e| worker::Error::from(e.to_string()))?;

            let body = res.text().await.map_err(|e| worker::Error::from(e.to_string()))?;
            Response::ok(body)
        })
        .run(req, env)
        .await?
        .with_cors(&cors)
}
