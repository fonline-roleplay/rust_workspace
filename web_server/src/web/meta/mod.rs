use crate::web::AppState;
use actix_http::h1::MessageType::Payload;
use actix_service::Service;
use actix_session::Session;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{
    error::{BlockingError, InternalError},
    http::header,
    web, HttpRequest, HttpResponse, Responder,
};
use futures::{
    future::{err as fut_err, ok as fut_ok, Either},
    Future, FutureExt, TryFutureExt,
};
use oauth2::{AsyncCodeTokenRequest, AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;

const DISCORD_CSRF_COOKIE_NAME: &str = "csrf_discord";
const DISCORD_USER_ID_COOKIE_NAME: &str = "user_id_discord";
const DISCORD_API_URL: &str = "https://discordapp.com/api";

mod auth;
mod rank;

pub use self::{
    auth::auth,
    rank::{extract_rank, get_ranks, get_user_record, Rank},
};

fn bad_request(text: &'static str) -> impl Fn() -> InternalError<&'static str> {
    move || {
        let full_text = format!("Bad request: {:?}", text);
        InternalError::from_response(
            text,
            HttpResponse::BadRequest()
                .content_type("text/plain; charset=utf-8")
                .body(full_text),
        )
    }
}

pub fn access_denied(text: &'static str) -> impl Fn() -> InternalError<&'static str> {
    move || {
        let full_text = format!("Access denied: {:?}", text);
        InternalError::from_response(
            text,
            HttpResponse::Forbidden()
                .content_type("text/plain; charset=utf-8")
                .body(full_text),
        )
    }
}

fn internal_error<D: std::fmt::Debug>(err: D) -> actix_web::Error {
    let text = format!("Internal error: {:?}", err);
    InternalError::from_response(
        text.clone(),
        HttpResponse::BadRequest()
            .content_type("text/plain; charset=utf-8")
            .body(text),
    )
    .into()
}

pub fn get_user_id(session: &Session) -> Option<u64> {
    match session.get(DISCORD_USER_ID_COOKIE_NAME) {
        Ok(Some(uid)) => Some(uid),
        Err(err) => {
            eprintln!("get_user_id error: {:?}", err);
            session.remove(DISCORD_USER_ID_COOKIE_NAME);
            None
        }
        Ok(None) => None,
    }
}

pub fn restrict_gm<
    S: Service<Response = ServiceResponse, Request = ServiceRequest, Error = actix_web::Error>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = Result<ServiceResponse, actix_web::Error>> {
    let first_rank = extract_rank(&req);
    let mut fut = srv.call(req);
    async move {
        if first_rank? >= Rank::GameMaster {
            fut.await
        } else {
            Err(access_denied("Rank is too low for this restricted zone")())?
        }
    }
}

pub async fn login(
    //path: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let (authorize_url, csrf_token) = data
        .oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_owned()))
        .url();
    session.remove(DISCORD_USER_ID_COOKIE_NAME);
    session.set(DISCORD_CSRF_COOKIE_NAME, csrf_token)?;
    /*println!(
        "Login: session: {:?}",
        session.get::<String>(DISCORD_CSRF_COOKIE_NAME)
    );*/
    Ok(HttpResponse::Found()
        .header(header::LOCATION, authorize_url.to_string())
        .header(header::ACCESS_CONTROL_MAX_AGE, "0")
        .finish())
}

pub async fn logout(session: Session) -> actix_web::Result<HttpResponse> {
    session.remove(DISCORD_USER_ID_COOKIE_NAME);
    Ok(HttpResponse::Found()
        .header(header::LOCATION, "/")
        .header(header::ACCESS_CONTROL_MAX_AGE, "0")
        .finish())
}
