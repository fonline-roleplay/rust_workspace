//use crate::webserver;
use lazy_static::lazy_static;
use tnf_common::engine_types::critter::Critter;

//lazy_static! {
//    static ref WEBSERVER: webserver::Mailbox = { webserver::run() };
//}

#[no_mangle]
pub extern "C" fn launch_actix() {
    //let _ = *WEBSERVER;
}

#[no_mangle]
pub extern "C" fn update_character(cr: &Critter) {
    //if let Err(err) = WEBSERVER.update_critter(cr) {
    //    eprintln!("Error updating character: {:?}", err);
    //}
}
