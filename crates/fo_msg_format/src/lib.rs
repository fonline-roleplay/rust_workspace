mod lexer;

use std::collections::btree_map::{self, BTreeMap};

#[derive(Debug, PartialEq)]
pub struct MsgDictionary {
    index_to_string: BTreeMap<(u32, u32), Box<str>>,
}

impl MsgDictionary {
    fn new() -> Self {
        Self {
            index_to_string: BTreeMap::new(),
        }
    }
    pub fn get_first(&self, index: u32) -> Option<&str> {
        self.index_to_string.get(&(index, 0)).map(AsRef::as_ref)
    }
    pub fn get_all(&self, index: u32) -> impl Iterator<Item = (u32, &str)> {
        self.index_to_string
            .range((index, 0)..(index, u32::MAX))
            .map(|(&(_index, sub_index), value)| (sub_index, value.as_ref()))
    }
    pub fn insert(&mut self, index: u32, value: Box<str>) {
        let sub_index = self
            .index_to_string
            .range((index, 0)..(index, u32::MAX))
            .last()
            .map(|((_index, sub_index), _value)| sub_index + 1)
            .unwrap_or(0);
        let old = self.index_to_string.insert((index, sub_index), value);
        assert_eq!(old, None);
    }
    pub fn iter_firsts(&self) -> impl Iterator<Item = (u32, &str)> {
        self.index_to_string
            .iter()
            .filter_map(|(&(index, sub_index), value)| {
                if sub_index == 0 {
                    Some((index, value.as_ref()))
                } else {
                    None
                }
            })
    }
}

#[derive(Debug, PartialEq)]
struct Msg<'a> {
    lines: Vec<Line<'a>>,
}

#[derive(Debug, PartialEq)]
enum Line<'a> {
    Entry(Entry<'a>),
    Break,
    Comment(&'a str),
}

#[derive(Debug, PartialEq)]
struct Entry<'a> {
    index: u32,
    secondary: &'a str,
    value: &'a str,
    comment: Option<&'a str>,
}

fn parse_msg(input: &str) -> Result<MsgDictionary, String> {
    let msg = lexer::tokenize_msg(input, true)?;
    let mut dict = MsgDictionary::new();
    for line in msg.lines {
        match line {
            Line::Entry(entry) => {
                if !entry.secondary.is_empty() {
                    panic!("Non-empty secondary key! {:?}", entry);
                }
                dict.insert(entry.index, entry.value.into())
            }
            Line::Break | Line::Comment(_) => {
                //ignore line breaks and comments
            }
        }
    }
    Ok(dict)
}

#[cfg(any(test, feature = "cp1251"))]
pub fn parse_cp1251_file<P: AsRef<std::path::Path>>(path: P) -> Result<MsgDictionary, String> {
    let bytes = std::fs::read(path).map_err(|err| format!("IoError: {}", err))?;
    use encoding_rs::*;
    let (cow, encoding_used, had_errors) = WINDOWS_1251.decode(&bytes);
    if encoding_used != WINDOWS_1251 {
        return Err(format!(
            "Wrong decoding used: {:?}, should be: {:?}",
            encoding_used, WINDOWS_1251
        ));
    }
    if had_errors {
        return Err("CP1251 decoding error".into());
    }
    //println!("{:?}", cow.as_ref());
    parse_msg(&cow)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sample() {
        const SAMPLE: &str = "\
            # Transit Name, (pid + 1) * 10 + 8 pm added\n\
            \n\
            # Map 0, Global, base 10\n\
            {10}{}{Global map}\n\
            {15}{}{20car}\n\
            {15}{}{23world}\n\
            {15}{}{03 - A Way To Anywhere.ogg}\
        ";
        let dict = parse_msg(SAMPLE).unwrap();
        let correct = mock_dict(&[
            ((10, 0), "Global map"),
            ((15, 0), "20car"),
            ((15, 1), "23world"),
            ((15, 2), "03 - A Way To Anywhere.ogg"),
        ]);
        assert_eq!(dict, correct);
    }

    fn mock_dict(data: &[((u32, u32), &str)]) -> MsgDictionary {
        let mut dict = MsgDictionary::new();
        for &((index, sub_index), value) in data {
            dict.index_to_string
                .insert((index, sub_index), value.into());
        }
        dict
    }

    fn file_id<P: AsRef<std::path::Path>>(file: P) -> Option<String> {
        let file = file.as_ref();
        Some(format!(
            "{}/{}",
            file.parent()?.file_name()?.to_str()?,
            file.file_name()?.to_str()?,
        ))
    }

    #[test]
    fn parse_all_forp_msg_files() {
        let mut vec = vec![];
        for dir in &["../../../FO4RP/text/eng", "../../../FO4RP/text/russ"] {
            for file in std::fs::read_dir(dir).unwrap() {
                let path = file.unwrap().path();
                if let Some(ext) = path.extension() {
                    if &ext.to_str().unwrap().to_uppercase() == "MSG" {
                        //if path.file_name().unwrap() == "FOGM.MSG" {
                        let path_str = path.to_str().unwrap();
                        let res = parse_cp1251_file(&path).expect(path_str);
                        vec.push((file_id(path).unwrap(), res));
                    }
                }
            }
        }
        /*for (file, dict) in vec {
            for ((index, sub_index), value) in &dict.index_to_string {
                println!("[{}][{}][{}] \"{}\"", file, index, sub_index, value);
            }
        }*/
    }
}
