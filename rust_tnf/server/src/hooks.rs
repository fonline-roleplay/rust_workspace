use std::{
    sync::{
        mpsc::{channel, Sender, Receiver},
        Mutex,
    },
    cell::Cell,
};
use tnf_common::{
    engine_types::critter::{Critter, CritterInfo},
};
use crate::webserver;
use lazy_static::lazy_static;

thread_local! {
    pub static ACTIX: Cell<Option<Sender<CritterInfo>>> = Cell::new(None);
}

lazy_static!{   
    static ref ACTIX_GLOBAL: Mutex<(Sender<CritterInfo>, Option<Receiver<CritterInfo>>)> = {
        let (sender, reciever) = channel();
        Mutex::new((sender, Some(reciever)))
    };
}

#[no_mangle]
pub extern "C" fn launch_actix() {
    println!("Actix launch called from {:?}", ::std::thread::current().id());
    let (sender, reciever) = &mut *ACTIX_GLOBAL.lock().expect("Can't lock mutex.");
    let reciever = reciever.take().expect("Can launch actix only once!");
    ACTIX.with(|actix| actix.set(Some(sender.clone())));
    ::std::thread::spawn(|| {
        webserver::run(reciever);
    });
}

#[no_mangle]
pub extern "C" fn update_character(cr: &Critter) {
    println!("Update character called from {:?}", ::std::thread::current().id());
    ACTIX.with(|actix| {
        let sender = if let Some(sender) = actix.take() {
            sender
        } else {
            let (sender, _) = &*ACTIX_GLOBAL.lock().expect("Can't lock mutex.");
            sender.clone()
        };
        let _res = sender.send(CritterInfo::new(cr));
        actix.set(Some(sender));
    });
}