use super::*;
use crate::{
    database::{
        ownership::{get_auth, get_ownership, set_ownership},
        VersionedError,
    },
    utils::blocking,
    web::avatar,
};

enum AuthAction {
    CheckOwnership { user_id: u64, char_id: u32 },
    //CheckAuthKey(super::avatar::AuthVec),
    Auth,
}

fn extract_auth(req: &HttpRequest) -> Option<avatar::AuthVec> {
    let method = req.method();
    if method == &Method::GET {
        let query: web::Query<avatar::Auth> = web::Query::from_query(req.query_string()).ok()?;
        let (auth, _auth_string) = avatar::parse_auth(&query)?;
        Some(auth)
    } else {
        None
    }
}
async fn restrict_client_inner(req: &HttpRequest) -> Result<AuthAction, actix_web::Error> {
    let url_id = req
        .match_info()
        .get("id")
        .and_then(|id| id.parse::<u32>().ok())
        .ok_or_else(access_denied("Wrong id"))?;
    let method = req.method().clone();
    let member = extract_member(req);

    if let Some(member) = member.await? {
        return match member.ranks.first() {
            Some(rank) if rank >= &Rank::Player => Ok(AuthAction::CheckOwnership {
                user_id: member.id,
                char_id: url_id,
            }),
            _ => Err(access_denied("Rank is too low for this restricted zone")().into()),
        };
    }

    if method == &Method::GET {
        Ok(AuthAction::Auth)
    } else {
        Err(access_denied("Restricted zone")().into())
    }
    /*
    let auth_vec = extract_auth(req).ok_or_else(access_denied("Restricted zone"))?;
    Ok(AuthAction::CheckAuthKey(auth_vec))*/
}

/*pub fn restrict_ownership<
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = Result<ServiceResponse, actix_web::Error>>
where
    S::Future: 'static,
{
    let action = match restrict_client_inner(&req) {
        // Logged but has no role, or unlogged wih unsuitable method for redirect
        Err(err) => {
            return Either::Left(fut_err(err));
        }
        Ok(action) => action,
    };

    let data: &web::Data<AppState> = req.app_data().expect("Can't happend here");

    Either::Right(match action {
        // Unlogged, redirect to login
        AuthAction::Auth => {
            let session = req.get_session();
            if let Some(path_and_query) = req.uri().path_and_query() {
                session
                    .set(LOCATION_AFTER_AUTH, path_and_query.as_str())
                    .expect("String can't fail serialization");
            }

            Either::Left(
                login(data.clone(), session).map_ok(move |response| req.into_response(response)),
            )

            //Service::call(login, &req).await
        }
        // Logged and has role, checking for ownership
        AuthAction::CheckOwnership { user_id, char_id } => {
            let root = data.sled_db.root.clone();
            let auth_received = extract_auth(&req);
            let fut = srv.call(req);
            let right = async move {
                let result: Result<(), VersionedError> = blocking(move || {
                    let owner = get_ownership(&root, char_id)?;
                    dbg!(&owner);
                    match owner {
                        Some(owner) if owner == user_id => {
                            return Ok(());
                        }
                        None => {}
                        _ => {
                            return Err(VersionedError::AccessDenied);
                        }
                    }

                    let auth_stored = get_auth(&root, char_id)?;
                    dbg!(&auth_stored);
                    match (owner, auth_stored, auth_received) {
                        (None, Some(auth_stored), Some(auth_received))
                            if &*auth_stored == &*auth_received =>
                        {
                            set_ownership(&root, char_id, user_id)?;
                            Ok(())
                        }
                        _ => Err(VersionedError::AccessDenied),
                    }
                })
                .await;
                match result {
                    Ok(()) => fut.await,
                    Err(err) => Err(err.into()),
                }
            };
            Either::Right(right)
        }
    })
}
*/

pub async fn restrict_ownership(req: HttpRequest) -> Result<Restrict, actix_web::Error> {
    let action = restrict_client_inner(&req).await?;

    let data: &web::Data<AppState> = req.app_data().expect("Can't happend here");

    match action {
        // Unlogged, redirect to login
        AuthAction::Auth => {
            let session = req.get_session();
            if let Some(path_and_query) = req.uri().path_and_query() {
                session
                    .insert(LOCATION_AFTER_AUTH, path_and_query.as_str())
                    .expect("String can't fail serialization");
            }

            login(data.clone(), session)
                .map_ok(move |response| Restrict::response(req, response))
                .await
        }
        // Logged and has role, checking for ownership
        AuthAction::CheckOwnership { user_id, char_id } => {
            let root = data.sled_db.root.clone();
            let auth_received = extract_auth(&req);
            let result: Result<(), VersionedError> = blocking(move || {
                let owner = get_ownership(&root, char_id)?;
                dbg!(&owner);
                match owner {
                    Some(owner) if owner == user_id => {
                        return Ok(());
                    }
                    None => {}
                    _ => {
                        return Err(VersionedError::AccessDenied);
                    }
                }

                let auth_stored = get_auth(&root, char_id)?;
                dbg!(&auth_stored);
                match (owner, auth_stored, auth_received) {
                    (None, Some(auth_stored), Some(auth_received))
                        if &*auth_stored == &*auth_received =>
                    {
                        set_ownership(&root, char_id, user_id)?;
                        Ok(())
                    }
                    _ => Err(VersionedError::AccessDenied),
                }
            })
            .await;
            // TODO: Refactor
            Ok(match result {
                Ok(()) => Restrict::Allow,
                Err(err) => Restrict::Deny(format!("Access denied: {:?}", err)),
            })
        }
    }
}
