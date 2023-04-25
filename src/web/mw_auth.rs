#[allow(unused)]
use crate::ctx::Ctx;
use crate::model::ModelController;
use crate::web::AUTH_TOKEN_KEY;
use crate::{Error, Result};
use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum::http::request::Parts;
use axum::{http::Request, middleware::Next, response::Response};
use axum::{RequestExt, RequestPartsExt};
use lazy_regex::regex_captures;
use tower_cookies::{Cookie, Cookies};

pub async fn mv_require_auth<B>(
    ctx: Result<Ctx>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    println!("->> {:<12} - mw_require_auth - {ctx:?}", "MIDDLEWARE");
    ctx?;
    Ok(next.run(req).await)
}

pub async fn mv_ctx_resolver<B>(
    _mc: State<ModelController>,
    cookies: Cookies,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<Response> {
    println!("->> {:<12} - mv_ctx_resolver", "MIDDLEWARE");
    let auth_token = cookies.get(AUTH_TOKEN_KEY).map(|c| c.value().to_string());

    // Compute Result<Ctx>
    let result_ctx = match auth_token
        .ok_or(Error::AuthFailNoAuthTokenCookie)
        .and_then(parse_token) //当前值是 `Some` 或 `Ok` 时执行一个闭包
    {
        Ok((user_id, _exp, _sign)) => {
            // TODO token validate
            Ok(Ctx::new(user_id))
        }
        Err(e) => Err(e),
    };
    // Remove the cookie if something went wrong other than NoAuthtokenCookie.
    // 除了没有cookie之外的错误都移除cookie
    if result_ctx.is_err() && !matches!(result_ctx, Err(Error::AuthFailNoAuthTokenCookie)) {
        cookies.remove(Cookie::named(AUTH_TOKEN_KEY));
    }

    // store the ctx_result to request extension
    req.extensions_mut().insert(result_ctx);

    Ok(next.run(req).await)
}

// region: --- helper functions
//如果您的提取器需要使用请求主体，那么您应该实现FromRequest 而不是FromRequestParts。
#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for Ctx {
    //如果提取器失败，它将使用这种“拒绝”类型。拒绝是一种可以转化为响应的错误。
    type Rejection = Error;
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        println!("->> {:<12} - Ctx", "EXTRACTOR");
        // Extract cookies from request
        // 中间件把ctx插入到extensions,在此获取
        parts
            .extensions
            .get::<Result<Ctx>>()
            .ok_or(Error::AuthFailCtxNotInRequestExt)?
            .clone()
    }
}

// endregion

//parse a token of format `user-[user-id].[expiration].[signature]`
//returns (user_id,expiration,signature)
pub fn parse_token(token: String) -> Result<(u64, String, String)> {
    let (_whole, user_id, exp, sign) = regex_captures!(r#"^user-(\d+)\.(.+)\.(.+)"#, &token)
        .ok_or(Error::AuthFailTokenWrongFormat)?;

    let user_id = user_id
        .parse::<u64>()
        .map_err(|_| Error::AuthFailTokenWrongFormat)?;
    Ok((user_id, exp.to_string(), sign.to_string()))
}
