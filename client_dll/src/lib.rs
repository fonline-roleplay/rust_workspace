#![cfg(windows)]

use tnf_common::engine_types::{ScriptArray, ScriptString};

//#[cfg(debug_assertions)]
tnf_common::dll_main!({});

#[no_mangle]
pub extern "C" fn open_link(link: &ScriptString) {
    let link = link.string();
    if link.starts_with("http://") || link.starts_with("https://") {
        std::thread::spawn(move || {
            #[cfg(debug_assertions)]
            println!("Opening link: {:?}", link);
            let _res = webbrowser::open(&link);
        });
    } else {
        println!("Invalid link: {:?}", link);
    }
}

fn send_avatars(avatars: Vec<u8>) -> std::io::Result<()> {
    use std::io::Write;
    let mut stream = std::net::TcpStream::connect_timeout(
        &"127.0.0.1:33741".parse().unwrap(),
        std::time::Duration::from_micros(50),
    )?;
    stream.write_all(&avatars[..])
}

#[no_mangle]
pub extern "C" fn update_avatars(array: &ScriptArray) {
    //let points: Option<&[i32]> = array.cast();
    let buffer = array.buffer();
    //if let Some(points) = points {
    //let vec = points.to_owned();
    let vec = buffer.to_owned();
    std::thread::spawn(move || {
        send_avatars(vec);
    });
    //}
}
