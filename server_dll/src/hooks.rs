//use crate::webserver;
use crate::bridge;
use lazy_static::lazy_static;
use std::io::Write;
use tnf_common::engine_types::critter::Critter;
use tnf_common::engine_types::{ScriptArray, ScriptString};

//lazy_static! {
//    static ref WEBSERVER: webserver::Mailbox = { webserver::run() };
//}

#[no_mangle]
pub extern "C" fn init_compat(offset: usize) {
    crate::engine_functions::init(offset);
}

#[no_mangle]
pub extern "C" fn main_loop() {
    //bridge::init();
    let messages = bridge::receive();
    for message in messages {
        use crate::{
            engine_functions::{get_critter, run_client_script},
            param::change_uparam,
        };
        use bridge::MsgIn;
        use tnf_common::defines::param::Param;
        match message {
            MsgIn::UpdateCharLeaf { id, ver, secret } => {
                if let Some(cr) = get_critter(id) {
                    if !change_uparam(cr, Param::QST_CHAR_VER, ver) {
                        eprintln!("Can't notify about parameter change!");
                    }
                    if !change_uparam(cr, Param::QST_CHAR_SECRET, secret) {
                        eprintln!("Can't notify about parameter change!");
                    }
                }
            }
            MsgIn::SendKeyToPlayer(cr_id, key) => {
                if let Some(cr) = get_critter(cr_id) {
                    run_client_script(
                        cr,
                        "link@OpenWithKey",
                        key[0] as i32,
                        key[1] as i32,
                        key[2] as i32,
                    );
                }
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn critter_init(cr: &Critter, first_time: bool) {
    if !cr.CritterIsNpc {
        println!("Critter is player: {}", cr.Id);
        bridge::send_one(bridge::MsgOut::PlayerConnected(cr.Id));
    }
}

#[no_mangle]
pub extern "C" fn player_auth(cr: &Critter) {
    if !cr.CritterIsNpc {
        println!("Player requesting auth: {}", cr.Id);
        bridge::send_one(bridge::MsgOut::PlayerAuth(cr.Id));
    } else {
        eprintln!("Critter is not player: {}!", cr.Id);
    }
}

#[no_mangle]
pub extern "C" fn update_character(cr: &Critter) {
    //if let Err(err) = WEBSERVER.update_critter(cr) {
    //    eprintln!("Error updating character: {:?}", err);
    //}
}
/*
#[no_mangle]
pub extern "C" fn rust_check_critter(critter_id: u32) -> u32 {
    println!("test");

    let cr = crate::engine_functions::get_critter(critter_id);
    let cr_id = cr.map(|cr| cr.Id);
    println!("id: {:?}", cr_id);
    cr_id.unwrap_or(0)
}
*/
//# pragma bindfunc "uint CheckCritter(uint) -> rust_dll/server.dll rust_check_critter"
