use clients_db::ClientsDb;

use itertools::EitherOrBoth;
use itertools::Itertools;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn main() {
    /*
    let ids = ClientsDb::list_ids("../../FO4RP/save/clients/".into());
    for (id, path) in &ids {
        let name = path.file_name().expect("file name").to_str().expect("file name str");
        if name.starts_with("ё") {
            let mut to = path.clone();
            to.set_file_name(format!("{}", *id));
            to.set_extension("client");
            dbg!(&path);
            dbg!(&to);
            std::fs::rename(&path,&to).expect("rename");
        }
    }
    */
    //let names = ClientsDb::list_names("../../FO4RP_move/save/clients/".into());
    /*
    let join: BTreeMap<u32, (&PathBuf, &String)> = ids.iter()
        .merge_join_by(names.iter(), |(id1, _path), (id2, _name)| id1.cmp(id2))
        .filter_map(|either| {
            match either {
                EitherOrBoth::Both(a, b) => {
                    Some((*a.0, (a.1, b.1)))
                },
                _ => None
            }
        })
        .filter(|(id, (path,name))| {
            path.file_stem().unwrap().to_str().unwrap() != name.as_str()
        })
        .collect();
    */
    //println!("Совпадений: {:?}", join.len());
    //println!("{:#?}", join);
    /*for (key, (path, name)) in join.iter() {
        let mut to = (*path).clone();
        to.set_file_name(name);
        to.set_extension("client");
        std::fs::rename(&path,&to).unwrap();
    }*/
    //ClientsDb::fix_clients("../../FO4RP_move/save/clients/".into(), false);
}
/*
use tnf_web_server::*;

fn main() -> std::io::Result<()> {
    let clients = env::expect_clients();
    env::setup_working_dir()?;

    let not_test = std::env::args().nth(1).as_ref().map(AsRef::as_ref) == Some("--notest");
    critters_db::CrittersDb::fix_clients(clients, !not_test);

    Ok(())
}
*/
