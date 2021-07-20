//use crate::webserver;
use crate::bridge;
use cstr::cstr;
use tnf_common::engine_types::{ScriptArray, ScriptString, critter::Critter};
use tnf_common::{
    state::State,
    engine_types::{
        game_options::{game_state},
    },
};
//lazy_static! {
//    static ref WEBSERVER: webserver::Mailbox = { webserver::run() };
//}

//const FUNC_LINK_OPEN_WITH_KEY: &'static CStr =
//    unsafe { CStr::from_bytes_with_nul_unchecked(b"link@OpenWithKey\0") };

#[no_mangle]
pub extern "C" fn main_loop() {
    use crate::{
        engine_functions::{get_critter, run_client_script, run_critter_script},
        param::change_uparams,
    };
    //bridge::init();
    let messages = bridge::receive();
    for message in messages {
        use bridge::MsgIn;
        use tnf_common::defines_fo4rp::param::Param;
        println!("{:?}", message);
        match message {
            MsgIn::UpdateCharLeaf { id, ver, secret } => {
                if let Some(cr) = get_critter(id) {
                    if !change_uparams(
                        cr,
                        &[(Param::QST_CHAR_VER, ver), (Param::QST_CHAR_SECRET, secret)],
                    ) {
                        eprintln!("Can't notify about parameter change!");
                    }
                }
            }
            MsgIn::SendKeyToPlayer(cr_id, key) => {
                if let Some(cr) = get_critter(cr_id) {
                    #[allow(bad_style)]
                    let FUNC_LINK_OPEN_WITH_KEY = cstr!("link@OpenWithKey");
                    if let Err(err) = run_client_script(
                        cr,
                        FUNC_LINK_OPEN_WITH_KEY,
                        key[0] as i32,
                        key[1] as i32,
                        key[2] as i32,
                        None,
                        None,
                    ) {
                        eprintln!("MsgIn::SendKeyToPlayer! cr_id: {:?}, err: {:?}", cr_id, err);
                    }
                }
            }
            MsgIn::SendConfig { player_id, url } => {
                if let Some(player) = get_critter(player_id) {
                    #[allow(bad_style)]
                    let FUNC_LINK_UPDATE_URL = cstr!("link@UpdateUrl");
                    if let Err(err) =
                        run_client_script(player, FUNC_LINK_UPDATE_URL, 0, 0, 0, Some(&url), None)
                    {
                        eprintln!(
                            "MsgIn::SendConfig! player_id: {:?}, err: {:?}",
                            player_id, err
                        );
                    }
                }
            }
            MsgIn::StartGame { player_id } => {
                if let Some(player) = get_critter(player_id) {
                    if let Err(err) = run_critter_script(
                        Some(player),
                        cstr!("map_start@StartGame"),
                        0,
                        0,
                        0,
                        None,
                        None,
                    ) {
                        eprintln!(
                            "MsgIn::StartGame! player_id: {:?}, err: {:?}",
                            player_id, err
                        );
                    }
                }
            }
            MsgIn::Nop => {
                eprintln!("Msg::In received, probably bug");
            }
        }
    }

    let status = crate::Server::with(|server| {
        let connections = server.statistics_connections();
        let day_time = bridge::DayTime::from_hour(game_state().unwrap().Hour);
        bridge::ServerStatus{connections, day_time}
    });
    bridge::send_one(bridge::MsgOut::Status(status));
}

#[no_mangle]
pub extern "C" fn critter_init(cr: &Critter, first_time: bool) {
    if !cr.CritterIsNpc {
        println!("Critter is player: {}", cr.Id);
        bridge::send_one(bridge::MsgOut::PlayerConnected(cr.Id));
    }
}

#[no_mangle]
pub extern "C" fn player_login(ip: u32, name: &ScriptString, id: u32) -> bool {
    true
}

#[no_mangle]
pub extern "C" fn player_after_login(player: &Critter) {
    bridge::send_one(bridge::MsgOut::PlayerConnected(player.Id));
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
pub extern "C" fn discord_send_message(channel: Option<&ScriptString>, text: Option<&ScriptString>) {
    if let Some(msg) = (move || {
        let channel = channel?.string();
        let text = text?.string();
        Some(bridge::MsgOut::DiscordSendMessage{channel, text})
    })() {
        bridge::send_one(msg);
    }
}

#[no_mangle]
pub extern "C" fn prev_hex(cr: &Critter, hex_x: &mut u16, hex_y: &mut u16) -> u32 {
    *hex_x = cr.PrevHexX;
    *hex_y = cr.PrevHexY;
    cr.PrevHexTick
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
