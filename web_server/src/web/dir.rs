use actix_files::Directory;
use actix_web::{dev::ServiceResponse, HttpRequest, HttpResponse};
use percent_encoding::{utf8_percent_encode, CONTROLS};
use std::fs::DirEntry;
use std::{fmt::Write, io, path::Path};
use v_htmlescape::escape as escape_html_entity;

// show file url as relative to static path
macro_rules! encode_file_url {
    ($path:ident) => {
        utf8_percent_encode(&$path, CONTROLS)
    };
}

// " -- &quot;  & -- &amp;  ' -- &#x27;  < -- &lt;  > -- &gt;  / -- &#x2f;
macro_rules! encode_file_name {
    ($entry:ident) => {
        escape_html_entity(&$entry.file_name().to_string_lossy())
    };
}

fn directory_listing(dir: &Directory, req: &HttpRequest) -> Result<ServiceResponse, io::Error> {
    let index_of = format!("Index of {}", req.path());
    let mut body = String::new();
    let base = Path::new(req.path());

    let mut entries = dir
        .path
        .read_dir()?
        .filter_map(|entry| entry.ok())
        //.map(|res| res.map(|e| e.path()))
        .collect::<Vec<DirEntry>>();
    entries.sort_by_cached_key(|entry| entry.file_name());

    for entry in entries {
        if is_visible(&entry) {
            let p = match entry.path().strip_prefix(&dir.path) {
                Ok(p) if cfg!(windows) => base.join(p).to_string_lossy().replace("\\", "/"),
                Ok(p) => base.join(p).to_string_lossy().into_owned(),
                Err(_) => continue,
            };

            // if file is a directory, add '/' to the end of the name
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let _ = write!(
                        body,
                        "<li><a href=\"{}\">{}/</a></li>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                    );
                } else {
                    let _ = write!(
                        body,
                        "<li><a href=\"{}\">{}</a></li>",
                        encode_file_url!(p),
                        encode_file_name!(entry),
                    );
                }
            } else {
                continue;
            }
        }
    }

    let html = format!(
        "<html>\
         <head><title>{}</title></head>\
         <body><h1>{}</h1>\
         <ul>\
         {}\
         </ul></body>\n</html>",
        index_of, index_of, body
    );
    Ok(ServiceResponse::new(
        req.clone(),
        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(html),
    ))
}

pub fn is_visible(entry: &DirEntry) -> bool {
    if let Some(name) = entry.file_name().to_str() {
        if name.starts_with('.') {
            return false;
        }
    }
    if let Ok(ref md) = entry.metadata() {
        let ft = md.file_type();
        return ft.is_dir() || ft.is_file() || ft.is_symlink();
    }
    false
}
