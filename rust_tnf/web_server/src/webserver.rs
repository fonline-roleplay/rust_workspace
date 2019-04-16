use std::sync::mpsc::channel;

use actix::prelude::{Actor, Addr, SendError, SyncArbiter};
use actix_web::{fs, http, server, App, Error, HttpRequest, HttpResponse, Responder};
use futures::{future::ok as fut_ok, future::Either, Future};

use tnf_common::engine_types::critter::Critter;

use crate::{
    critter_info::CritterInfo,
    critters_db::{
        ClientRecord, CrittersDb, GetClientInfo, GetCritterInfo, ListClients, UpdateCritterInfo,
    },
};

const STATIC_PATH: &'static str = "./static/";

mod stats;

pub struct Mailbox(actix::Addr<CrittersDb>);
impl Mailbox {
    pub fn update_critter(&self, cr: &Critter) -> Result<(), SendError<UpdateCritterInfo>> {
        self.0
            .try_send(UpdateCritterInfo::from(CritterInfo::from(cr)))
    }
}

fn nope(_req: &HttpRequest<AppState>) -> impl Responder {
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
    file: &'a str,
}
impl<'a> ClientsList<'a> {
    fn new<I: Iterator<Item = (&'a String, &'a ClientRecord)>>(clients: I) -> Self {
        Self {
            clients: clients
                .map(|(name, record)| ClientRow {
                    name: &name,
                    file: record.filename.to_str().unwrap_or(""),
                })
                .collect(),
        }
    }
    fn render(&self) -> Result<String, templates::TemplatesError> {
        templates::render("gm_clients.html", self)
    }
}

fn gm_clients(req: &HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    req.state()
        .critters_db
        .send(ListClients)
        .from_err()
        .and_then(|res| match res {
            Ok(clients) => match ClientsList::new(clients.iter()).render() {
                Ok(body) => Ok(HttpResponse::Ok().content_type("text/html").body(body)),
                Err(err) => {
                    eprintln!("GM Clients error: {:#?}", err);
                    Ok(HttpResponse::InternalServerError().into())
                }
            },
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
}

fn _info(req: &HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    let crid = req
        .match_info()
        .get("crid")
        .and_then(|crid| crid.parse().ok());
    if let Some(crid) = crid {
        Either::A(
            req.state()
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
}

impl AppState {
    pub fn new(critters_db: Addr<CrittersDb>) -> Self {
        Self { critters_db }
    }
}

pub fn run() {
    println!("Starting actix-web server...");

    let sys = actix::System::new("charsheet");

    crate::templates::init();

    //let addr = CrittersDb::start_default();
    let addr = SyncArbiter::start(1, || CrittersDb::new());

    let state = AppState::new(addr.clone());
    server::HttpServer::new(move || {
        App::with_state(state.clone())
            .resource("/", |r| r.method(http::Method::GET).f(nope))
            .resource("/gm/clients", |r| r.method(http::Method::GET).a(gm_clients))
            .resource("/gm/client/{client}", |r| {
                r.method(http::Method::GET).a(stats::gm_stats)
            })
            .resource("/{crid}", |r| r.method(http::Method::GET).a(stats::stats))
            .handler(
                "/static",
                fs::StaticFiles::new(STATIC_PATH)
                    .unwrap()
                    .show_files_listing(),
            )
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .start(); //.expect("Can't start server!");

    println!("Server started!");
    let _ = sys.run();
}
