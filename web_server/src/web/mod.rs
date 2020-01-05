use futures::{
    future::{ok as fut_ok, Either},
    Future,
};

use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::sync::Arc;

use crate::{bridge, config, critters_db::CrittersDb, database::SledDb};
use fo_data::FoData;

const STATIC_PATH: &'static str = "./static/";

mod avatar;
mod data;
mod gm;
mod map_viewer;
mod meta;
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

async fn index(
    _req: HttpRequest,
    session: actix_session::Session,
    data: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    let body = match meta::get_user_id(&session) {
        Some(user_id) => {
            let (name_string, max_rank) = match meta::get_user_record(&*data, user_id) {
                Ok(record) => (
                    match &record.nick {
                        Some(nick) => format!(r#"{} ({})"#, record.name, nick),
                        None => format!(r#"{}"#, record.name),
                    },
                    record.ranks.first().cloned(),
                ),
                Err(err) => (
                    {
                        eprintln!("Index page error: {}", err);
                        format!(r#"<red>error</red>"#)
                    },
                    None,
                ),
            };
            let menu = max_rank
                .filter(|rank| *rank >= meta::Rank::GameMaster)
                .map_or(String::new(), |_| {
                    format!(
                        "<h1>Menu:</h1><ul>\
                         <li><a href=\"gm/clients\">clients</a></li>\
                         <li><a href=\"private/\">private</a></li>\
                         <li><a href=\"maps\">maps</a></li>\
                         </ul>"
                    )
                });
            format!(
                r#"User: {} <a href="/meta/logout">Logout</a>{}"#,
                name_string, menu
            )
        }
        None => format!(r#"<a href="/meta/login">Login</a>"#),
    };
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body))
}

async fn go_web(data: web::Data<AppState>) -> HttpResponse {
    let url = data.config.host.web_url("/");
    HttpResponse::MovedPermanently()
        .header(actix_http::http::header::LOCATION, url)
        .finish()
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
    oauth: oauth2::basic::BasicClient,
    config: config::Config,
    mrhandy: Arc<mrhandy::MrHandy>,
    sled_db: SledDb,
    critters_db: CrittersDb,
    bridge: bridge::Bridge,
    fo_data: Arc<FoData>,
    items: Arc<BTreeMap<u16, fo_proto_format::ProtoItem>>,
    reqwest: reqwest::Client,
}

impl AppState {
    pub fn new(
        config: config::Config,
        mrhandy: mrhandy::MrHandy,
        db: sled::Db,
        fo_data: FoData,
        items: BTreeMap<u16, fo_proto_format::ProtoItem>,
    ) -> Self {
        let critters_db = CrittersDb::new(config.paths.save_clients.clone());

        let sled_db = SledDb::new(db);
        let bridge = bridge::Bridge::new();

        let redirect = config.host.web_url("/meta/auth");
        let oauth = oauth2_client(&config.discord.oauth2, redirect).expect("oauth client");

        let reqwest = reqwest::Client::builder()
            // Following redirects opens the client up to SSRF vulnerabilities.
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(2))
            .use_rustls_tls()
            .build()
            .expect("Build reqwest client");

        Self {
            oauth,
            config,
            mrhandy: Arc::new(mrhandy),
            sled_db,
            critters_db,
            bridge,
            fo_data: Arc::new(fo_data),
            items: Arc::new(items),
            reqwest,
        }
    }
    fn start_bridge(&self) {
        self.bridge.start(self.sled_db.root.clone());
    }
}

use actix_service::Service;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{
    http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use std::collections::BTreeMap;

pub fn oauth2_client(
    config: &config::OAuth,
    redirect: String,
) -> Result<oauth2::basic::BasicClient, Box<dyn std::error::Error>> {
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
    let client = BasicClient::new(
        ClientId::new(config.client_id.clone()),
        Some(ClientSecret::new(config.secret.clone())),
        AuthUrl::new("https://discordapp.com/api/oauth2/authorize".to_string())?,
        Some(TokenUrl::new(
            "https://discordapp.com/api/oauth2/token".to_string(),
        )?),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_url(RedirectUrl::new(redirect)?);
    Ok(client)
}

pub fn run(state: AppState) {
    println!("Starting actix-web server...");

    let sys = actix_rt::System::new("charsheet");

    crate::templates::init();

    state.start_bridge();

    let config = state.config.clone();

    let web_server = HttpServer::new({
        let state = state.clone();
        move || {
            let cookies = actix_session::CookieSession::private(state.config.session.cookie_key())
                .name("meta-session")
                .secure(false);
            App::new()
                .data(state.clone())
                .wrap(middleware::Compress::default())
                .wrap(middleware::Logger::default())
                .wrap_fn(restrict_web)
                .wrap(cookies)
                .service(web::resource("/").route(web::get().to(index)))
                .service(
                    web::scope("/meta")
                        .service(web::resource("/login").route(web::get().to(meta::login)))
                        .service(web::resource("/logout").route(web::get().to(meta::logout)))
                        .service(web::resource("/auth").route(web::get().to(meta::auth))),
                )
                .service(
                    web::scope("/maps")
                        .wrap_fn(meta::restrict_gm)
                        //.service(web::resource("/tilemap").route(web::get().to(map_viewer::tilemap))),
                        .service(web::resource("/{path:.+}").route(web::get().to(map_viewer::view)))
                        .service(web::resource("").route(web::get().to(map_viewer::list))),
                )
                .service(
                    web::scope("/gm")
                        .wrap_fn(meta::restrict_gm)
                        .service(web::resource("/clients").route(web::get().to(gm::clients)))
                        .service(
                            web::resource("/client/{client}").route(web::get().to(stats::gm_stats)),
                        ),
                )
                .service(
                    web::resource("/data/{path:.+}")
                        .wrap_fn(meta::restrict_gm)
                        .route(web::get().to(data::get)),
                )
                .service(actix_files::Files::new("/static", STATIC_PATH))
                .service(
                    web::scope("/char/{id}")
                        .service(
                            web::scope("/edit").service(
                                web::resource("/avatar")
                                    .route(web::get().to(avatar::edit))
                                    .route(web::post().to(avatar::upload)),
                            ),
                        )
                        .service(web::resource("/avatar").route(web::get().to(avatar::show))),
                )
                .service({
                    let mut private = web::scope("/private")
                        .wrap_fn(meta::restrict_gm)
                        .service(web::resource("/").route(web::get().to(list_privates)));
                    let name_path = state.config.paths.privates();
                    for (name, path) in name_path {
                        private = private.service(
                            actix_files::Files::new(&format!("/{}", name), path)
                                .show_files_listing(),
                        );
                    }
                    private
                })
            //.service(
            //    web::resource("/{crid}").route(web::get().to_async(stats::gm_stats))
            //)
        }
    })
    .server_hostname(config.host.web.domain_port());

    let web_port = config.host.web_port();
    if let Some(cert) = &config.host.web_tls {
        let tls_config = cert.server_config().expect("TLS server config");
        web_server.bind_rustls(("0.0.0.0", web_port), tls_config)
    } else {
        web_server.bind(("0.0.0.0", web_port))
    }
    .expect("Can't bind web server to port")
    .run();

    HttpServer::new(move || {
        App::new()
            .data(state.clone())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            //.wrap_fn(restrict_files)
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("Access-Control-Allow-Origin", state.config.host.web_url("")),
            )
            .service(web::resource("/").route(web::get().to(go_web)))
            .service(actix_files::Files::new("/static", STATIC_PATH))
            .service(
                web::scope("/char/{id}")
                    .service(
                        web::scope("/edit").service(
                            web::resource("/avatar")
                                .route(web::get().to(avatar::edit))
                                .route(web::post().to(avatar::upload)),
                        ),
                    )
                    .service(web::resource("/avatar").route(web::get().to(avatar::show))),
            )
    })
    .server_hostname(config.host.files.domain_port())
    .bind(("0.0.0.0", config.host.files_port()))
    .expect("Can't bind files server to port")
    .run();

    println!("Servers started!");
    let _ = sys.run();
}

fn req_host(req: &ServiceRequest) -> Option<&str> {
    let mut host = req.uri().host();
    //println!("uri host: {:?}", host);
    if host.is_none() {
        host = req
            .headers()
            .get(actix_http::http::header::HOST)
            .and_then(|header| header.to_str().ok())
            .and_then(|host_port| host_port.split(':').next());
        //println!("headers host: {:?}", host);
    }
    host
}

fn restrict_web<
    B: MessageBody,
    S: Service<Response = ServiceResponse<B>, Request = ServiceRequest, Error = actix_web::Error>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = Result<ServiceResponse<B>, actix_web::Error>> {
    let data: web::Data<AppState> = req.app_data().expect("AppData");
    let host = req_host(&req);
    let check = host.map_or(false, |host| host == data.config.host.web.domain);
    let mut fut = srv.call(req);
    async move {
        if check {
            fut.await
        } else {
            Err(meta::access_denied("Wrong domain name")())?
        }
    }
}

fn restrict_files<
    B: MessageBody,
    S: Service<Response = ServiceResponse<B>, Request = ServiceRequest, Error = actix_web::Error>,
>(
    req: ServiceRequest,
    srv: &mut S,
) -> impl Future<Output = Result<ServiceResponse<B>, actix_web::Error>> {
    let data: web::Data<AppState> = req.app_data().expect("AppData");
    let host = req.headers().get("host");
    let host = req_host(&req);
    let check = host.map_or(false, |host| host == data.config.host.files.domain);
    let mut fut = srv.call(req);
    async move {
        if check {
            fut.await
        } else {
            Err(meta::access_denied("Wrong domain name")())?
        }
    }
}

async fn list_privates(data: web::Data<AppState>) -> HttpResponse {
    let name_path = data.config.paths.privates();
    let body: String = name_path
        .keys()
        .map(|name| format!(r#"<p><a href="{0}">{0}</a></p>"#, name))
        .collect();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(body)
}
