use super::{web, AppState, HttpResponse};
use crate::{
    config::Host,
    database::{ownership::get_ownership, Root},
    templates,
};
use clients_db::{fix_encoding::os_str_debug, ClientRecord};
use fo_defines::CritterParam;
use fo_defines_fo4rp::{fos, param::Param};
use futures::{Future, FutureExt};
use serde::Serialize;
use std::{borrow::Cow, net::Ipv4Addr, time::Duration};

pub async fn clients(data: web::Data<AppState>) -> actix_web::Result<HttpResponse> {
    let mrhandy = data.mrhandy.as_ref().expect("Discord config");
    let members = mrhandy.clone_members().await;
    let res = web::block(move || {
        let clients = data.critters_db.list_clients();
        let list = ClientsList::new(
            clients.clients().iter(),
            &data.sled_db.root,
            members.as_ref(),
        );
        list.render(&data.config.host)
    })
    .await?
    .map_err(|err| super::internal_error(err))?;

    Ok(HttpResponse::Ok().content_type("text/html").body(res))
}

#[derive(Debug, Serialize)]
struct ClientsList<'a> {
    clients: Vec<ClientRow<'a>>,
}
#[derive(Debug, Serialize)]
struct ClientRow<'a> {
    name: &'a str,
    file: Cow<'a, str>,
    info: Option<ClientRowInfo<'a>>,
    last_seen: Option<(String, bool)>,
}
#[derive(Debug, Serialize)]
struct ClientRowInfo<'a> {
    id: u32,
    lvl: i32,
    hp: i32,
    map_id: u32,
    map_pid: u16,
    cond: &'static str,
    st_access_level: i32,
    qst_vision: i32,
    gamemode: &'static str,
    discord: Result<OwnerInfo<'a>, &'static str>,
    ip: &'a [Ipv4Addr],
}

const GAMEMODS: [&'static str; fos::GAME_MAX as usize] =
    ["START", "ADVENTURE", "SURVIVAL", "ARCADE", "TEST"];

fn get_name<'a>(
    members: Option<&'a mrhandy::Members>,
    root: &Root,
    id: u32,
) -> Result<OwnerInfo<'a>, &'static str> {
    let members = members.ok_or("Main guild unavaible")?;
    let owner = get_ownership(root, id)
        .map_err(|_| "Err")?
        .ok_or("No owner")?;
    let member = match members.get(owner) {
        Some(member) => member,
        None => return Ok(OwnerInfo::Id(owner)),
    };
    let name = member.user.name.as_str();
    Ok(match member.nick.as_ref() {
        None => OwnerInfo::Name(name),
        Some(nick) => OwnerInfo::NickName(name, nick.as_str()),
    })
}

#[derive(Debug, Serialize)]
enum OwnerInfo<'a> {
    Id(u64),
    Name(&'a str),
    NickName(&'a str, &'a str),
}

impl<'a> ClientsList<'a> {
    fn new<I: Iterator<Item = (&'a String, &'a ClientRecord)>>(
        clients: I,
        root: &Root,
        members: Option<&'a mrhandy::Members>,
    ) -> Self {
        Self {
            clients: clients
                .map(|(name, record)| {
                    let info = record.info.as_ref().map(|info| ClientRowInfo {
                        id: info.id,
                        lvl: info.param(Param::ST_LEVEL),
                        hp: info.param(Param::ST_CURRENT_HP),
                        map_id: info.map_id,
                        map_pid: info.map_pid,
                        cond: info.cond(),
                        st_access_level: info.param(Param::ST_ACCESS_LEVEL),
                        qst_vision: info.param(Param::QST_VISION),
                        gamemode: GAMEMODS
                            [info.uparam(Param::QST_GAMEMODE).min(fos::GAME_MAX - 1) as usize],
                        discord: get_name(members, root, info.id), //.unwrap_or_else(|err| Cow::Borrowed(err)),
                        ip: &info.ip[..],
                    });
                    ClientRow {
                        info,
                        name: &name,
                        file: os_str_debug(&record.filename),
                        last_seen: record
                            .modified
                            .and_then(|time| time.elapsed().ok())
                            .as_ref()
                            .map(ago),
                    }
                })
                .collect(),
        }
    }
    fn render(&self, host: &Host) -> Result<String, templates::TemplatesError> {
        templates::render("gm_clients.html", self, host)
    }
}

fn ago(duration: &Duration) -> (String, bool) {
    let secs = duration.as_secs();
    (
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 60 * 60 {
            format!("{}m", secs / 60)
        } else if secs < 24 * 60 * 60 {
            format!("{}h", secs / 60 / 60)
        } else {
            format!("{}d", secs / 60 / 60 / 24)
        },
        secs < 60 * 5,
    )
}
