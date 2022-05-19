use crate::web::{internal_error, restrict::Restrict, AppState};
use actix_session::Session;
use actix_web::{
    error::InternalError,
    http::{header, Method},
    web, HttpRequest, HttpResponse,
};
use futures::{
    Future, TryFutureExt,
};
use oauth2::{AuthorizationCode, CsrfToken, Scope, TokenResponse};
use serde::Deserialize;

const DISCORD_CSRF_COOKIE_NAME: &str = "csrf_discord";
const DISCORD_USER_ID_COOKIE_NAME: &str = "user_id_discord";
const DISCORD_API_URL: &str = "https://discordapp.com/api";

const LOCATION_AFTER_AUTH: &str = "location_after_auth";

mod auth;
mod ownership;
mod rank;
pub use ownership::restrict_ownership;

pub use self::{
    auth::auth,
    rank::{extract_member, get_ranks, get_user_record, Rank},
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

pub async fn restrict_gm(req: HttpRequest) -> Result<Restrict, actix_web::Error> {
    let first_rank = extract_member(&req).await?;
    Ok(match first_rank {
        Some(member) => match member.ranks.first() {
            Some(rank) if rank >= &Rank::GameMaster => Restrict::Allow,
            _ => Restrict::Deny("Rank is too low for this restricted zone".into()),
        },
        None => Restrict::Deny("Restricted zone".into()),
    })
}

pub async fn login(
    //path: web::Path<String>,
    data: web::Data<AppState>,
    session: Session,
) -> actix_web::Result<HttpResponse> {
    let (authorize_url, csrf_token) = data
        .oauth
        .as_ref()
        .expect("OAuth config")
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("identify".to_owned()))
        .url();
    session.remove(DISCORD_USER_ID_COOKIE_NAME);
    session.insert(DISCORD_CSRF_COOKIE_NAME, csrf_token)?;
    /*println!(
        "Login: session: {:?}",
        session.get::<String>(DISCORD_CSRF_COOKIE_NAME)
    );*/
    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url.to_string()))
        .append_header((header::ACCESS_CONTROL_MAX_AGE, "0"))
        .finish())
}

pub async fn logout(session: Session) -> actix_web::Result<HttpResponse> {
    session.remove(DISCORD_USER_ID_COOKIE_NAME);
    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, "/"))
        .append_header((header::ACCESS_CONTROL_MAX_AGE, "0"))
        .finish())
}
