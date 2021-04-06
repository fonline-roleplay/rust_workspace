use super::*;
use actix_session::UserSession;
use actix_web::web::Data;
use std::sync::Arc;

pub async fn get_ranks(data: Arc<AppState>, user_id: u64) -> Result<Vec<Rank>, &'static str> {
    data.mrhandy
        .as_ref()
        .expect("Discord config")
        .with_guild_member(user_id, |guild, member| {
            let mut ranks = mrhandy::MrHandy::get_roles(
                guild,
                member,
                role_to_rank(&data.config.discord.as_ref().expect("Discord config").roles),
            );
            ranks.sort_by_key(|key| std::cmp::Reverse(*key));
            ranks
        })
        .await
}

pub async fn get_user_record(data: &AppState, user_id: u64) -> Result<UserRecord, &'static str> {
    data.mrhandy
        .as_ref()
        .expect("Discord config")
        .with_guild_member(user_id, |guild, member| {
            let mut ranks = mrhandy::MrHandy::get_roles(
                guild,
                member,
                role_to_rank(&data.config.discord.as_ref().expect("Discord config").roles),
            );
            ranks.sort_by_key(|key| std::cmp::Reverse(*key));
            let (name, nick) = mrhandy::MrHandy::get_name_nick(member);
            UserRecord { name, nick, ranks }
        })
        .await
}

pub struct UserRecord {
    pub name: String,
    pub nick: Option<String>,
    pub ranks: Vec<Rank>,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub enum Rank {
    Unknown,
    Player,
    GameMaster,
    Developer,
    Admin,
}

fn role_to_rank<'b>(config: &'b crate::config::Roles) -> impl 'b + Fn(&mrhandy::Role) -> Rank {
    move |role| {
        if role.name == config.player {
            Rank::Player
        } else if role.name == config.gamemaster {
            Rank::GameMaster
        } else if role.name == config.developer {
            Rank::Developer
        } else if role.name == config.admin {
            Rank::Admin
        } else {
            Rank::Unknown
        }
    }
}

pub struct Member {
    pub id: u64,
    pub ranks: Vec<Rank>,
}

pub fn extract_member(
    req: &HttpRequest,
) -> impl Future<Output = Result<Option<Member>, actix_web::Error>> {
    let data = req
        .app_data()
        .cloned()
        .map(Data::<AppState>::into_inner)
        .ok_or("No AppState data")
        .map_err(internal_error);
    let session = req.get_session();
    let user_id = get_user_id(&session);

    async move {
        match user_id {
            Some(id) => {
                let ranks = get_ranks(data?, id).await.map_err(internal_error)?;
                Ok(Some(Member { id, ranks }))
            }
            None => Ok(None),
        }
    }
}
