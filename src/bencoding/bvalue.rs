use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::from_utf8,
};

use crate::bytes_reader::BytesReader;

use super::{blist::BList, BDict, BInteger, BString};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum BValue<'a> {
    String(BString<'a>),
    Integer(BInteger<'a>),
    List(BList<'a>),
    Dict(BDict<'a>),
}

impl<'a> BValue<'a> {
    pub fn decode(encoded: &'a [u8]) -> Self {
        let mut reader = BytesReader::new(encoded);
        Self::parse(&mut reader)
    }

    fn parse(reader: &mut BytesReader<'a>) -> Self {
        match reader.peek() {
            b'd' => {
                reader.skip();
                let mut map = HashMap::new();
                while reader.peek() != b'e' {
                    let Self::String(key) = Self::parse(reader) else {
                        panic!("dictionary keys must be strings")
                    };
                    map.insert(key, Self::parse(reader));
                }
                reader.skip();
                BValue::Dict(map.into())
            }
            b'i' => {
                reader.skip();
                let integer = reader.read_until(b'e');
                reader.skip();
                BValue::Integer(integer.into())
            }
            b'l' => {
                reader.skip();
                let mut list = vec![];
                while reader.peek() != b'e' {
                    list.push(Self::parse(reader));
                }
                reader.skip();
                BValue::List(list.into())
            }
            c if c.is_ascii_digit() => {
                let len = from_utf8(reader.read_until(b':'))
                    .unwrap()
                    .parse::<usize>()
                    .unwrap();
                reader.skip();
                BValue::String(reader.read_range(len).into())
            }
            _ => {
                unimplemented!()
            }
        }
    }
}

impl<'a> Display for BValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl<'a> TryFrom<&'a BValue<'a>> for &BString<'a> {
    type Error = &'a BValue<'a>;
    fn try_from(value: &'a BValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BValue::String(bstring) => Ok(bstring),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a BValue<'a>> for &BInteger<'a> {
    type Error = &'a BValue<'a>;
    fn try_from(value: &'a BValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BValue::Integer(binteger) => Ok(binteger),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a BValue<'a>> for &BList<'a> {
    type Error = &'a BValue<'a>;
    fn try_from(value: &'a BValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BValue::List(blist) => Ok(blist),
            _ => Err(value),
        }
    }
}

impl<'a> TryFrom<&'a BValue<'a>> for &BDict<'a> {
    type Error = &'a BValue<'a>;
    fn try_from(value: &'a BValue<'a>) -> Result<Self, Self::Error> {
        match value {
            BValue::Dict(bdict) => Ok(bdict),
            _ => Err(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use serde_bencode;
    use serde_json::json;

    #[test]
    fn test_string_single_digit_len() {
        let val = "apple";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_string_multi_digit_len() {
        let val = "watermelon";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_string_utf8() {
        let val = "naïve";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        assert_eq!(BytesReader::new(&encoded).read(), b'6');
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_positive() {
        let val = 1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_negative() {
        let val = -1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_zero() {
        let val = 0;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list() {
        let val = ["spam", "eggs"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list_emtpy() {
        let val: [&str; 0] = [];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list_le_el() {
        let val = ["le", "el"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_dict() {
        let mut val = HashMap::new();
        val.insert("spam", vec!['a', 'b']);
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::decode(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_read_torrent_file() {
        let encoded = fs::read("sample.torrent").unwrap();
        let decoded = serde_json::to_string_pretty(&BValue::decode(&encoded)).unwrap();
        println!("{decoded}");
    }
}
