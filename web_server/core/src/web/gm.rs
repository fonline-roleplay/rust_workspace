use super::{web, AppState, HttpResponse};
use crate::{config::Host, templates, database::{Root, ownership::get_ownership}};
use clients_db::{fix_encoding::os_str_debug, ClientRecord, CritterInfo};
use fo_defines::CritterParam;
use fo_defines_fo4rp::{fos, param::Param};
use futures::{future as fut, Future, FutureExt};
use serde::Serialize;
use std::{borrow::Cow, net::Ipv4Addr, time::Duration};

pub fn clients(data: web::Data<AppState>) -> impl Future<Output = actix_web::Result<HttpResponse>> {
    web::block(move || -> Result<_, ()> { 
        let clients = data.critters_db.list_clients();
        let mrhandy = data.mrhandy.as_ref().expect("Discord config");
        let server = mrhandy.get_server();
        let server_read = server.as_ref().map(mrhandy::Server::read);
        match ClientsList::new(clients.clients().iter(), &data.sled_db.root, &server_read).render(&data.config.host) {
            Ok(body) => Ok(body),
            Err(err) => {
                eprintln!("Template error: {:?}", err);
                Err(())
            }
        }
    }).map(|res| match res {
        Ok(body) => {
            Ok(HttpResponse::Ok().content_type("text/html").body(body))
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().into())
        }
    })
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
    gamemode: &'static str,
    discord: Result<OwnerInfo<'a>, &'static str>,
    ip: &'a [Ipv4Addr],
}

const GAMEMODS: [&'static str; fos::GAME_MAX as usize] =
    ["START", "ADVENTURE", "SURVIVAL", "ARCADE", "TEST"];

fn get_name<'a, E>(server: &'a Result<mrhandy::ServerRead<'a>, E>, root: &Root, id: u32) -> Result<OwnerInfo<'a>, &'static str> {
    let server = server.as_ref().map_err(|_| "Err")?;
    let owner = get_ownership(root, id).map_err(|_| "Err")?.ok_or("None")?;
    let member = match server.get_member(owner) {
        Ok(member) => member,
        Err(_) => return Ok(OwnerInfo::Id(owner)),
    };
    let user_read = member.user.read();
    let name = &user_read.name;
    Ok(match member.nick.as_ref() {
        None => OwnerInfo::Name(name.clone()),
        Some(nick) => OwnerInfo::NickName(name.clone(), nick.as_str()),
    })
}

#[derive(Debug, Serialize)]
enum OwnerInfo<'a> {
    Id(u64),
    Name(String),
    NickName(String, &'a str)
}

impl<'a> ClientsList<'a> {
    fn new<E, I: Iterator<Item = (&'a String, &'a ClientRecord)>>(clients: I, root: &Root, server_read: &'a Result<mrhandy::ServerRead<'a>, E>) -> Self {
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
                        gamemode: GAMEMODS
                            [info.uparam(Param::QST_GAMEMODE).min(fos::GAME_MAX - 1) as usize],
                        discord: get_name(server_read, root, info.id), //.unwrap_or_else(|err| Cow::Borrowed(err)),
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
