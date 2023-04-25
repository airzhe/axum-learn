use crate::log::log_request;
use crate::model::ModelController;

#[allow(unused)]
// 导出使其可以在其他模块使用
pub use self::error::{Error, Result};
use std::net::SocketAddr;

use axum::http::{status, Method, Uri};
use axum::response::Response;
use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::{get, get_service},
};
use axum::{middleware, Json, Router};
use ctx::Ctx;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

mod ctx;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
    //`route_layer`是一种特殊类型的中间件，它只会应用于某个具体的路由（route）
    //`layer`是一般的中间件类型，它可以应用于整个应用程序，
    // init modelcontroller.
    let mc = ModelController::new().await?;
    let routes_apis = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mv_require_auth));

    let routes_all = Router::new()
        .merge(router_hello())
        .merge(web::router_login::routes())
        .nest("/api", routes_apis)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mv_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    // region: run it with hyper on localhost:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("->> LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // endregion
    Ok(())
}
async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();
    let server_error = res.extensions().get::<Error>();
    //if client_status_error build new response
    let client_status_error = server_error.map(|se| se.client_status_and_error());
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error" :{
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            println!("  ->> client_error_body:  {client_error_body}");
            (*status_code, Json(client_error_body)).into_response()
        });
    println!("  ->> server log line - {uuid} - Error: {server_error:?}");

    //Build and log the server log line
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, server_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}

// region: --- Static serve
fn routes_static() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("static")))
}
// endregion

// region: --- Routes Hello
fn router_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello2))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}

async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {params:?}", "HANDLER");
    //`as_deref()` 可以将 `Option<String>` 转换为 `Option<&str>`，这样无论 `params.name` 是哪种类型，最终都可以得到一个 `Option<&str>` 类型的值，便于后续处理。
    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello <strong>{name}</Strong>!"))
}

async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
    println!("->> {:<12} - handler_hello - {name}", "HANDLER");
    Html(format!("Hello <strong>{name}</Strong>!"))
}
// endregion
