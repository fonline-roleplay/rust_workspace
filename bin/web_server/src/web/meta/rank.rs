use super::*;
use actix_session::UserSession;

pub fn get_ranks(data: &AppState, user_id: u64) -> Result<Vec<Rank>, &'static str> {
    data.mrhandy.with_guild_member(user_id, |guild, member| {
        let mut ranks =
            mrhandy::MrHandy::get_roles(guild, member, role_to_rank(&data.config.discord.roles));
        ranks.sort_by_key(|key| std::cmp::Reverse(*key));
        ranks
    })
}

pub fn get_user_record(data: &AppState, user_id: u64) -> Result<UserRecord, &'static str> {
    data.mrhandy.with_guild_member(user_id, |guild, member| {
        let mut ranks =
            mrhandy::MrHandy::get_roles(guild, member, role_to_rank(&data.config.discord.roles));
        ranks.sort_by_key(|key| std::cmp::Reverse(*key));
        let (name, nick) = mrhandy::MrHandy::get_name_nick(member);
        UserRecord { name, nick, ranks }
    })
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

pub fn extract_rank(req: &ServiceRequest) -> Result<Rank, actix_web::Error> {
    use actix_session::{Session, UserSession};
    use actix_web::{web::Data, FromRequest};
    let data: Data<AppState> = req
        .app_data()
        .ok_or("No AppState data")
        .map_err(internal_error)?;
    let session = req.get_session();
    let user_id = get_user_id(&session).ok_or_else(access_denied("Restricted zone"))?;
    let ranks = get_ranks(&data, user_id).map_err(internal_error)?;
    let first_rank = ranks.first().ok_or_else(access_denied("Restricted zone"))?;
    Ok(*first_rank)
}
