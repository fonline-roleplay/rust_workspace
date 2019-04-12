use std::{
    sync::mpsc::Receiver,
};

use actix_web::{server, http, fs, App, HttpRequest, Responder, HttpResponse, Error};
use actix::prelude::{Addr, Actor, SyncArbiter, SyncContext};
use futures::{Future, future::ok as fut_ok, future::Either};

use tnf_common::{
    engine_types::critter::CritterInfo,
};

use crate::critters_db::{CrittersDb, GetCritterInfo};

mod stats;

fn nope(_req: &HttpRequest<AppState>) -> impl Responder {
    //let to = req.match_info().get("name").unwrap_or("World");
    format!("Hello there and go to hell!")
}

fn info(req: &HttpRequest<AppState>) -> impl Future<Item=HttpResponse, Error=Error> {
    let crid = req.match_info().get("crid").and_then(|crid| crid.parse().ok());
    if let Some(crid) = crid {
        Either::A(
            req.state().critters_db.send(GetCritterInfo{id: crid})
                .from_err()
                .and_then(|res| {
                    match res {
                        Ok(Some(cr_info)) => Ok(format!("Your id: {:?}", cr_info.Id).into()),
                        Ok(None) => Ok("I don't know about you!".into()),
                        Err(_) => Ok(HttpResponse::InternalServerError().into())
                    }
                })
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
        Self {
            critters_db,
        }
    }
}

fn oneshot_sync_actor<A>(actor: A) -> Addr<A>
    where A: Actor<Context = SyncContext<A>> + Send,
{
    SyncArbiter::start(1, {
        let crdb = std::sync::Mutex::new(Some(actor));
        move || {
            crdb.lock().expect("Can only run one CritterDb!").take().expect("Can only run one CritterDb!")
        }
    })
}

pub fn run(receiver: Receiver<CritterInfo>) {
    println!("Starting actix-web server...");
    let sys = actix::System::new("charlist");

    let addr = oneshot_sync_actor(CrittersDb::new(receiver));

    let state = AppState::new(addr);
    server::HttpServer::new(move || {
        App::with_state(state.clone())
            .resource("/", |r| r.method(http::Method::GET).f(nope))
            //.resource("/{crid}", |r| r.method(http::Method::GET).a(info))
            .resource("/{crid}", |r| r.method(http::Method::GET).a(stats::stats))
            .handler("/static", fs::StaticFiles::new("./web_static").unwrap().show_files_listing())
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .start(); //.expect("Can't start server!");

    println!("Server started!");
    let _ = sys.run();
}
