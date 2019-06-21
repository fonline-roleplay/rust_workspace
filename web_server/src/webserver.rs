use futures::{
    future::{ok as fut_ok, Either},
    Future,
};
use std::net::Ipv4Addr;
use std::path::PathBuf;
use std::sync::Arc;
use std::{borrow::Cow, sync::mpsc::channel, time::Duration};

use tnf_common::defines::{
    fos,
    param::{CritterParam, Param},
};

use clients_db::{fix_encoding::os_str_debug, ClientRecord, CritterInfo};

use crate::{
    bridge,
    critters_db::{CrittersDb, GetClientInfo, GetCritterInfo, ListClients, UpdateCritterInfo},
    database::SledDb,
};

const STATIC_PATH: &'static str = "./static/";

mod avatar;
mod stats;

/*
pub struct Mailbox(actix::Addr<CrittersDb>);
impl Mailbox {
    pub fn update_critter(&self, cr: &Critter) -> Result<(), SendError<UpdateCritterInfo>> {
        self.0
            .try_send(UpdateCritterInfo::from(CritterInfo::from(cr)))
    }
}
*/

fn nope(_req: HttpRequest) -> impl Responder {
    //let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello there and go to hell!")
}

use crate::templates;
use serde::Serialize;
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
    ip: &'a [Ipv4Addr],
}

const GAMEMODS: [&'static str; fos::GAME_MAX as usize] =
    ["START", "ADVENTURE", "SURVIVAL", "ARCADE", "TEST"];

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

impl<'a> ClientsList<'a> {
    fn new<I: Iterator<Item = (&'a String, &'a ClientRecord)>>(clients: I) -> Self {
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
    fn render(&self) -> Result<String, templates::TemplatesError> {
        templates::render("gm_clients.html", self)
    }
}

fn gm_clients(data: web::Data<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    data.get_ref()
        .critters_db
        .send(ListClients)
        .from_err()
        .and_then(|res| match res {
            Ok(clients) => match ClientsList::new(clients.clients().iter()).render() {
                Ok(body) => Ok(HttpResponse::Ok().content_type("text/html").body(body)),
                Err(err) => {
                    eprintln!("GM Clients error: {:#?}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
            },
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn _info(
    req: HttpRequest,
    data: web::Data<AppState>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    let crid = req
        .match_info()
        .get("crid")
        .and_then(|crid| crid.parse().ok());

    if let Some(crid) = crid {
        Either::A(
            data.get_ref()
                .critters_db
                .send(GetCritterInfo { id: crid })
                .from_err()
                .and_then(|res| match res {
                    Ok(Some(cr_info)) => Ok(format!("Your id: {:?}", cr_info.id).into()),
                    Ok(None) => Ok("I don't know about you!".into()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                }),
        )
    } else {
        Either::B(fut_ok("Get out!".into()))
    }
}

#[derive(Clone)]
pub struct AppState {
    critters_db: Addr<CrittersDb>,
    sled_db: SledDb,
    bridge: bridge::Bridge,
}

impl AppState {
    pub fn new(critters_db: Addr<CrittersDb>, sled_db: SledDb, bridge: bridge::Bridge) -> Self {
        Self {
            critters_db,
            sled_db,
            bridge,
        }
    }
}

use actix::prelude::{Actor, Addr, SendError, SyncArbiter};
use actix_web::{
    http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};

pub fn run(clients: PathBuf, db: sled::Db) {
    println!("Starting actix-web server...");

    let sys = actix::System::new("charsheet");

    crate::templates::init();

    //let addr = CrittersDb::start_default();
    let addr = SyncArbiter::start(1, move || CrittersDb::new(clients.clone()));

    let sled_db = SledDb::new(db);
    let bridge = bridge::start();

    let state = AppState::new(addr.clone(), sled_db, bridge);
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(nope)))
            .service(
                web::scope("/gm")
                    .service(web::resource("/clients").route(web::get().to_async(gm_clients)))
                    .service(
                        web::resource("/client/{client}")
                            .route(web::get().to_async(stats::gm_stats)),
                    ),
            )
            .service(actix_files::Files::new("/static", STATIC_PATH))
            .service(
                web::scope("/char/{id}")
                    .service(
                        web::scope("/edit").service(
                            web::resource("/avatar")
                                .route(web::get().to(avatar::edit))
                                .route(web::post().to_async(avatar::upload)),
                        ),
                    )
                    .service(web::resource("/avatar").route(web::get().to_async(avatar::show))),
            )
        //.service(
        //    web::resource("/{crid}").route(web::get().to_async(stats::gm_stats))
        //)
    })
    .bind("0.0.0.0:8000")
    .expect("Can not bind to port 8000")
    .start(); //.expect("Can't start server!");

    println!("Server started!");
    let _ = sys.run();
}
