mod lexer;

use nom_prelude::section;
use std::collections::{
    btree_map::{self, BTreeMap},
    HashMap,
};

#[derive(PartialEq, Debug)]
pub struct LstDictionary {
    index_to_string: BTreeMap<u32, String>,
    string_to_index: HashMap<String, u32>,
}

impl LstDictionary {
    pub fn index_to_string(&self, index: u32) -> Option<&str> {
        self.index_to_string.get(&index).map(String::as_str)
    }
    pub fn string_to_index(&self, string: &str) -> Option<u32> {
        self.string_to_index.get(string).copied()
    }
    pub fn iter(&self) -> impl Iterator<Item = (u32, &str)> {
        self.index_to_string
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
    }
}

#[derive(Debug, PartialEq)]
struct Lst<'a> {
    lines: Vec<Line<'a>>,
}

#[derive(Debug, PartialEq)]
enum Line<'a> {
    Section(u32),
    Entry(Entry<'a>),
    Break,
    Comment(&'a str),
}

#[derive(Debug, PartialEq)]
struct Entry<'a> {
    add: u32,
    name: &'a str,
}

pub fn parse_file<P: AsRef<std::path::Path>>(path: P) -> Result<LstDictionary, String> {
    let input = std::fs::read_to_string(path).map_err(|err| format!("IoError: {}", err))?;
    parse(&input)
}

pub fn parse(input: &str) -> Result<LstDictionary, String> {
    let lst = lexer::tokenize_lst(input, true)?;

    let mut index_to_string = BTreeMap::new();
    let mut string_to_index = HashMap::new();
    let mut current_section = None;
    for line in &lst.lines {
        match line {
            Line::Section(section) => current_section = Some(section),
            Line::Entry(entry) => {
                let base = current_section.copied().unwrap_or(0);
                let key = base.checked_add(entry.add).ok_or_else(|| {
                    format!("Overflow adding section {} and index {}", base, entry.add)
                })?;
                match index_to_string.entry(key) {
                    btree_map::Entry::Vacant(vacant) => {
                        let name = entry.name.to_owned();
                        vacant.insert(name.clone());
                        string_to_index.insert(name, key);
                    }
                    _ => {
                        // ignore entries with keys that was already inserted before
                        // probably deprecated stuff
                    }
                }
            }
            Line::Break | Line::Comment(_) => {
                //ignore line breaks and comments
            }
        }
    }
    Ok(LstDictionary {
        index_to_string,
        string_to_index,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_all_forp_lst() {
        parse_file_expect("../../../FO4RP/data/ItemNames.lst");
        parse_file_expect("../../../FO4RP/data/DefineNames.lst");
        parse_file_expect("../../../FO4RP/data/ParamNames.lst");
    }
    fn parse_file_expect(path: &str) {
        let _lst = parse_file(path).expect(path);
        //dbg!(&_lst);
    }
    #[test]
    fn parse_lst() {
        let input = "\
            0      ST_STRENGTH\n\
            1      ST_PERCEPTION\n\
            2      ST_ENDURANCE\n\
            \n\
            *200\n\
            0      SK_SMALL_GUNS\r\n\
            1      SK_BIG_GUNS\r\n\
            2      SK_ENERGY_WEAPONS\r\n\
            3      SK_UNARMED\r\n\
            # Deprecated\n\
            # Deprecated again\r\n\
            0      BT_MEN\
        ";
        let should_be = fill_maps(&[
            (0, "ST_STRENGTH"),
            (1, "ST_PERCEPTION"),
            (2, "ST_ENDURANCE"),
            (200, "SK_SMALL_GUNS"),
            (201, "SK_BIG_GUNS"),
            (202, "SK_ENERGY_WEAPONS"),
            (203, "SK_UNARMED"),
        ]);
        assert_eq!(parse(input).unwrap(), should_be);
    }
    fn fill_maps(data: &[(u32, &str)]) -> LstDictionary {
        let mut index_to_string = BTreeMap::new();
        let mut string_to_index = HashMap::new();
        for &(key, value) in data {
            index_to_string.insert(key, value.to_owned());
            string_to_index.insert(value.to_owned(), key);
        }
        LstDictionary {
            index_to_string,
            string_to_index,
        }
    }
}
