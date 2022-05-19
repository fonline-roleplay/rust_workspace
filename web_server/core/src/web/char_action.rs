use crate::bridge::MsgOut;
use actix_web::{web, Error, HttpResponse};

pub async fn start_game(
    path: web::Path<u32>,
    data: web::Data<super::AppState>,
) -> Result<HttpResponse, Error> {
    Ok(if let Some(_) = data
        .bridge
        .get_sender()
        .and_then(|mut sender| sender.try_send(MsgOut::StartGame { player_id: *path }).ok())
    {
        HttpResponse::Ok().body("Request to start game sent.")
    } else {
        HttpResponse::InternalServerError().body("Lost connection to game server.")
    })
}
