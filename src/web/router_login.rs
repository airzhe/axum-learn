use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::json;
use tower_cookies::Cookies;

use crate::{Error, Result, web};

pub fn routes() -> Router {
    Router::new().route("/api/login", post(api_login))
}

async fn api_login(
    cookies: Cookies,
    payload: Json<LoginPayload>,
) -> Result<Json<serde_json::Value>> {
    println!("->> {:<12} - api_login", "HANDLER");

    // todo implement real db/auth logic.
    if payload.username != "demo1" && payload.pwd == "welcome" {
        return Err(Error::LoginFail);
    }

    cookies.add(tower_cookies::Cookie::new(
        web::AUTH_TOKEN_KEY,
        "user-1.exp.sign",
    ));

    // create the success body.
    let body = Json(json!({
        "result": {
            "success": true,
        }
    }));
    Ok(body)
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}