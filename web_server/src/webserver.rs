use futures::{
    future::{ok as fut_ok, Either},
    Future,
};

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

use crate::{bridge, critters_db::CrittersDb, database::SledDb};

const STATIC_PATH: &'static str = "./static/";

mod avatar;
mod gm;
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

/*
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
*/
#[derive(Clone)]
pub struct AppState {
    critters_db: CrittersDb,
    sled_db: SledDb,
    bridge: bridge::Bridge,
}

impl AppState {
    pub fn new(critters_db: CrittersDb, sled_db: SledDb, bridge: bridge::Bridge) -> Self {
        Self {
            critters_db,
            sled_db,
            bridge,
        }
    }
}

use actix_web::{
    http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};

pub fn run(clients: PathBuf, db: sled::Db) {
    println!("Starting actix-web server...");

    let sys = actix_rt::System::new("charsheet");

    crate::templates::init();

    //let addr = CrittersDb::start_default();
    //let addr = SyncArbiter::start(1, move || CrittersDb::new(clients.clone()));
    let critter_db = CrittersDb::new(clients.clone());

    let sled_db = SledDb::new(db);
    let bridge = bridge::start(sled_db.root.clone());

    let state = AppState::new(critter_db, sled_db, bridge);
    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/").route(web::get().to(nope)))
            .service(
                web::scope("/gm")
                    .service(web::resource("/clients").route(web::get().to_async(gm::clients)))
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
                                .route(web::get().to_async(avatar::edit))
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
