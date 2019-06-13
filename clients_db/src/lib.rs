mod clients;
mod record;
mod critter_info;
pub mod fix_encoding;

pub use crate::{
    clients::ClientsDb,
    record::ClientRecord,
    critter_info::CritterInfo,
};

type InnerCritter = std::sync::Arc<CritterInfo>;
fn not_found() -> std::io::Error {
    std::io::ErrorKind::NotFound.into()
}
