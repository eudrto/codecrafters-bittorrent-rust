use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::from_utf8,
};

use crate::{bstring::BString, bytes_reader::BytesReader};

fn parse_int(bytes: &[u8]) -> i64 {
    let string = from_utf8(bytes).unwrap();
    string.parse::<i64>().unwrap()
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum BValue<'a> {
    String(BString<'a>),
    Integer(i64),
    List(Vec<BValue<'a>>),
    Dict(HashMap<BString<'a>, BValue<'a>>),
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
                BValue::Dict(map)
            }
            b'i' => {
                reader.skip();
                let integer = parse_int(reader.read_until(b'e'));
                reader.skip();
                BValue::Integer(integer)
            }
            b'l' => {
                reader.skip();
                let mut list = vec![];
                while reader.peek() != b'e' {
                    list.push(Self::parse(reader));
                }
                reader.skip();
                BValue::List(list)
            }
            c if c.is_ascii_digit() => {
                let len = parse_int(reader.read_until(b':')) as usize;
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

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use serde_bencode;
    use serde_json::json;

    #[test]
    fn test_parse_int() {
        let bytes = [b'4', b'2'];
        assert_eq!(parse_int(&bytes), 42);
    }

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
