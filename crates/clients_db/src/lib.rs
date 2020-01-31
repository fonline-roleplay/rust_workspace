mod clients;
mod critter_info;
pub mod fix_encoding;
mod record;

pub use crate::{clients::ClientsDb, critter_info::CritterInfo, record::ClientRecord};

type InnerCritter = std::sync::Arc<CritterInfo>;
fn not_found() -> std::io::Error {
    std::io::ErrorKind::NotFound.into()
}
