use std::sync::mpsc::channel;

use actix::prelude::{Actor, Addr, SendError};
use actix_web::{fs, http, server, App, Error, HttpRequest, HttpResponse, Responder};
use futures::{future::ok as fut_ok, future::Either, Future};

use tnf_common::engine_types::critter::{Critter, CritterInfo};

use crate::critters_db::{CrittersDb, GetCritterInfo, UpdateCritterInfo};

mod stats;

pub struct Mailbox(actix::Addr<CrittersDb>);
impl Mailbox {
    pub fn update_critter(&self, cr: &Critter) -> Result<(), SendError<UpdateCritterInfo>> {
        self.0
            .try_send(UpdateCritterInfo::from(CritterInfo::new(cr)))
    }
}

fn nope(_req: &HttpRequest<AppState>) -> impl Responder {
    //let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello there and go to hell!")
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
                    Ok(Some(cr_info)) => Ok(format!("Your id: {:?}", cr_info.Id).into()),
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

pub fn run() -> Mailbox {
    println!("Starting actix-web server...");

    let (sender, receiver) = channel();

    std::thread::spawn(move || {
        let sys = actix::System::new("charsheet");

        let addr = CrittersDb::start_default();

        let state = AppState::new(addr.clone());
        server::HttpServer::new(move || {
            App::with_state(state.clone())
                .resource("/", |r| r.method(http::Method::GET).f(nope))
                //.resource("/{crid}", |r| r.method(http::Method::GET).a(info))
                .resource("/{crid}", |r| r.method(http::Method::GET).a(stats::stats))
                .handler(
                    "/static",
                    fs::StaticFiles::new("./web_static")
                        .unwrap()
                        .show_files_listing(),
                )
        })
        .bind("127.0.0.1:8000")
        .expect("Can not bind to port 8000")
        .start(); //.expect("Can't start server!");

        sender
            .send(addr)
            .expect("Can't send CrittersDb address to engine's thread.");

        println!("Server started!");
        let _ = sys.run();
    });
    Mailbox(
        receiver
            .recv()
            .expect("Can't receive CrittersDb address from webserver thread."),
    )
}
