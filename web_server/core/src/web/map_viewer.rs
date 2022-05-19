use super::{web, AppState, HttpResponse};
use crate::{templates, utils::blocking};
use actix_web::error::BlockingError;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SpriteMap<'a> {
    min_x: i32,
    min_y: i32,
    tiles: Vec<Sprite<'a>>,
    objects: Vec<Sprite<'a>>,
}

#[derive(Debug, Serialize)]
struct Sprite<'a> {
    hex_x: u16,
    hex_y: u16,
    x: i32,
    y: i32,
    z: i32,
    path: &'a str,
}

pub async fn list() -> actix_web::Result<HttpResponse> {
    let res = blocking(|| -> Result<String, MapViewError> {
        let dir = std::fs::read_dir("../../FO4RP/maps").map_err(MapViewError::Io)?;
        let mut maps: Vec<_> = dir
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|entry| entry.path())
            .filter(|file| file.is_file() && file.extension() == Some("fomap".as_ref()))
            .collect();
        maps.sort();
        let response: String = maps
            .iter()
            .filter_map(|file| {
                file.file_name()
                    .and_then(|str| str.to_str())
                    .map(|name| format!("<p><a href = \"/maps/{0}\">{0}</a></p>\n", name))
            })
            .collect();
        Ok(response)
    }).await;
    Ok(match res {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(err) => {
            eprintln!("Map viewer error: {:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    })
}

#[derive(Debug)]
enum MapViewError {
    Io(std::io::Error),
    MapFormat(fo_map_format::Error),
    Nom(String),
    Template(templates::TemplatesError),
    Blocking,
}
impl From<BlockingError> for MapViewError {
    fn from(_err: BlockingError) -> Self {
        MapViewError::Blocking
    }
}

pub async fn view(
    path: web::Path<std::path::PathBuf>,
    data: web::Data<AppState>,
) -> actix_web::Result<HttpResponse> {
    use draw_geometry::fo as geometry;
    use primitives::Hex;
    let full_path = data.config.paths.maps.join(&*path);
    let res = blocking(move || {
        fo_map_format::verbose_read_file(full_path, |text, res| {
            let (_rest, map) =
                nom_prelude::nom_err_to_string(text, res).map_err(MapViewError::Nom)?;
            let mut min_x = i32::max_value(); //map.header.max_hex_x;
            let mut min_y = i32::max_value(); //map.header.max_hex_y;
            let tiles = map
                .tiles
                .0
                .iter()
                .filter(|tile| !tile.is_roof)
                .map(|tile| {
                    let (x, y) = (tile.hex_x as i32, tile.hex_y as i32);
                    let (x, y) = (
                        /*x = */ y * 16 - x * 24 - 24,
                        /*y = */ y * 12 + x * 6 + 24,
                    );
                    if x < min_x {
                        min_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    Sprite {
                        hex_x: tile.hex_x,
                        hex_y: tile.hex_y,
                        x,
                        y,
                        z: geometry::draw_order_pos_int(
                            geometry::DRAW_ORDER_FLAT + tile.layer.unwrap_or(0) as u32,
                            Hex::new(tile.hex_x, tile.hex_y),
                        )
                        .unwrap_or(0),
                        path: map
                            .tiles
                            .1
                            .to_path
                            .get(&tile.hash)
                            .expect("Hash must have related conventional path"),
                    }
                })
                .collect();
            let objects = map
                .objects
                .0
                .iter()
                .filter(|obj| obj.is_scenery())
                .filter_map(|obj| data.items.get(&obj.proto_id).map(|proto| (obj, proto)))
                .filter(|(_obj, proto)| {
                    (proto.Flags.unwrap_or(0) & fo_defines_fo4rp::fos::ITEM_HIDDEN) == 0
                })
                .map(|(obj, proto)| {
                    let (hex_x, hex_y) = (obj.map_x.unwrap_or(0), obj.map_y.unwrap_or(0));
                    let (x, y) = (hex_x as i32, hex_y as i32);
                    let (x, y) = (
                        /*x = */ y * 16 - x * 24 - (x % 2) * 8,
                        /*y = */ y * 12 + x * 6 - (x % 2) * 6,
                    );
                    if x < min_x {
                        min_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    Sprite {
                        hex_x,
                        hex_y,
                        x,
                        y,
                        z: geometry::draw_order_pos_int(
                            geometry::DrawOrderType::DRAW_ORDER_SCENERY as u32,
                            Hex::new(hex_x, hex_y),
                        )
                        .unwrap_or(0),
                        path: &proto.PicMap,
                    }
                })
                .collect();
            templates::render(
                "tilemap.html",
                &SpriteMap {
                    min_x,
                    min_y,
                    tiles,
                    objects,
                },
                templates::RenderConfig {
                    host: Some(&data.config.host),
                },
            )
            .map_err(MapViewError::Template)
        }, Default::default())
        .map_err(MapViewError::MapFormat)?
    }).await;
    Ok(match res {
        Ok(body) => HttpResponse::Ok().content_type("text/html").body(body),
        Err(err) => {
            eprintln!("Map viewer error: {:#?}", err);
            HttpResponse::InternalServerError().into()
        }
    })
}
