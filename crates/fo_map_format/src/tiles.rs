use crate::prelude::{complete::*, *};
use crc::crc32::checksum_ieee as crc32;
use std::cell::RefCell;

#[cfg(feature = "hashbrown")]
use hashbrown::HashMap;
#[cfg(not(feature = "hashbrown"))]
use std::collections::HashMap;

#[cfg(feature = "nohash-hasher")]
type HashMapU32<T> = HashMap<u32, T, nohash_hasher::BuildNoHashHasher<u32>>;
#[cfg(not(feature = "nohash-hasher"))]
type HashMapU32<T> = HashMap<u32, T>;

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
pub struct Tiles<'a>(
    #[cfg_attr(feature = "serde1", serde(borrow))] pub Vec<Tile<'a>>,
    pub Dict,
);

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
pub struct Tile<'a> {
    pub path: &'a str,
    pub hex_x: u16,
    pub hex_y: u16,
    pub offset: Option<(i8, i8)>,
    pub layer: Option<u8>,
    pub is_roof: bool,
    pub hash: u32,
}

#[cfg_attr(not(feature = "serde1"), derive(Debug))]
#[cfg_attr(feature = "serde1", derive(Serialize, Deserialize, SerDebug))]
#[derive(Default)]
pub struct Dict {
    // original path -> crc32
    pub to_hash: HashMap<String, u32>,
    // crc32 -> conventional path
    pub to_path: HashMapU32<String>,
}

fn tile<'a: 'b, 'b, E: ParseError<&'a str>>(
    dict: &'b RefCell<Dict>,
) -> impl 'b + Fn(&'a str) -> IResult<&'a str, Tile<'a>, E> {
    move |i| {
        let (i, is_roof) = alt((value(true, tag("roof")), value(false, tag("tile"))))(i)?;
        let (i, postfix) = opt(preceded(char('_'), pair(flag(char('o')), flag(char('l')))))(i)?;
        let (has_offset, has_layer) = postfix.unwrap_or((false, false));
        let (i, (hex_x, hex_y, offset, layer, path)) = tuple((
            space1_number,
            space1_number,
            cond(has_offset, pair(space1_number, space1_number)),
            cond(has_layer, space1_number),
            preceded(space1, word),
        ))(i)?;
        let mut dict = dict.borrow_mut();
        let hash = if let Some(&hash) = dict.to_hash.get(path) {
            hash
        } else {
            //let conventional_path = path.to_lowercase();
            //let conventional_path = conventional_path.replace('\\', "/");
            let conventional_path = make_path_conventional(path);
            let hash = crc32(conventional_path.as_bytes());
            dict.to_hash.insert(path.to_string(), hash);
            if let Some(first) = dict.to_path.insert(hash, conventional_path) {
                eprintln!(
                    "CRC32 collision? Different original paths? {:?} vs {:?}",
                    first,
                    dict.to_path.get(&hash)
                );
            }
            hash
        };

        Ok((
            i,
            Tile {
                path,
                hex_x,
                hex_y,
                offset,
                layer,
                is_roof,
                hash,
            },
        ))
    }
}

pub fn tiles<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Tiles<'a>, E> {
    let (i, _) = section("Tiles")(i)?;
    let dict = Default::default();
    let (i, tiles) = separated_list(t_rn, tile(&dict))(i)?;
    Ok((i, Tiles(tiles, dict.into_inner())))
}

#[cfg(test)]
mod test {
    //[16:860] Script callback: qwerty - 55151997 : main : void init() : 424, 2 : FOServer::InitReal : Game.
    //[16:860] Script callback: 123456 - 158520161 : main : void init() : 425, 2 : FOServer::InitReal : Game.
    //[16:860] Script callback: !@#$%^ - 3424808321 : main : void init() : 426, 2 : FOServer::InitReal : Game.

    //crc/crc32               time:   [14.046 ns 14.148 ns 14.271 ns]
    #[test]
    fn verify_crc32() {
        assert_eq!(crc::crc32::checksum_ieee(b"qwerty"), 55151997);
        assert_eq!(crc::crc32::checksum_ieee(b"123456"), 158520161);
        assert_eq!(crc::crc32::checksum_ieee(b"!@#$%^"), 3424808321);
    }

    //crc/crc32fast_new       time:   [30.002 ns 30.391 ns 30.889 ns]
    /*#[test]
    fn verify_crc32fast() {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(b"qwerty");
        assert_eq!(hasher.finalize(), 55151997);
        //assert_eq!(crc::crc32::checksum_ieee(b"123456"), 158520161);
        //assert_eq!(crc::crc32::checksum_ieee(b"!@#$%^"), 3424808321);
    }*/
}

impl crate::Offset for Tile<'_> {
    fn offset(&self) -> (i32, i32) {
        self.offset
            .map(|(x, y)| (x as i32, y as i32))
            .unwrap_or((0, 0))
    }
}
