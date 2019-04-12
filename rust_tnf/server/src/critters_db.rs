use actix::prelude::*;
use actix_web::{
    Error
};
use std::{
    sync::mpsc::{Receiver},
    collections::HashMap,
};
use tnf_common::engine_types::critter::CritterInfo;

pub struct CrittersDb {
    hashmap: HashMap<u32, CritterInfo>,
    receiver: Receiver<CritterInfo>,
}

impl CrittersDb {
    pub fn new(receiver: Receiver<CritterInfo>) -> Self {
        Self{
            receiver: receiver,
            hashmap: HashMap::new(),
        }
    }
}

impl Actor for CrittersDb {
    type Context = SyncContext<Self>;
}

pub struct GetCritterInfo {
    pub id: u32,
}

impl Message for GetCritterInfo {
    type Result = Result<Option<CritterInfo>, Error>;
}

impl Handler<GetCritterInfo> for CrittersDb {
    type Result = Result<Option<CritterInfo>, Error>;

    fn handle(&mut self, msg: GetCritterInfo, _: &mut Self::Context) -> Self::Result
    {
        for cr_info in self.receiver.try_iter() {
            let _old = self.hashmap.insert(cr_info.Id, cr_info);
        }
        Ok(self.hashmap.get(&msg.id).cloned())
    }
}