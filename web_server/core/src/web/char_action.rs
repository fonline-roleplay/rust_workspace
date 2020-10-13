use actix_web::{web, HttpResponse, Error};
use futures::Future;
use crate::bridge::MsgOut;

pub fn start_game(
    path: web::Path<u32>,
    data: web::Data<super::AppState>,
) -> impl Future<Output = Result<HttpResponse, Error>> {
    if let Some(sender) = data.bridge.get_sender()
        .and_then(|mut sender| {sender.try_send(MsgOut::StartGame{player_id: *path}).ok()})
    {
        HttpResponse::Ok().body("Request to start game sent.")
    } else {
        HttpResponse::InternalServerError().body("Lost connection to game server.")
    }
}
