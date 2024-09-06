use serde::{Serialize, Serializer};
use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    str::from_utf8,
};

fn peek(bytes: &[u8]) -> u8 {
    bytes[0]
}

fn advance(window: &mut &[u8]) {
    read(window);
}

fn read(window: &mut &[u8]) -> u8 {
    let result = peek(window);
    *window = &window[1..];
    result
}

fn read_range<'a>(window: &mut &'a [u8], len: usize) -> &'a [u8] {
    let result = &window[..len];
    *window = &window[len..];
    result
}

fn read_until<'a>(window: &mut &'a [u8], byte: u8) -> &'a [u8] {
    let pos = window.iter().position(|x| *x == byte).unwrap();
    let result = &window[..pos];
    *window = &window[pos + 1..];
    result
}

fn parse_int(bytes: &[u8]) -> i64 {
    let string = from_utf8(bytes).unwrap();
    string.parse::<i64>().unwrap()
}

#[derive(PartialEq, Eq, Hash)]
pub struct BString<'a>(&'a [u8]);

impl<'a> From<&'a [u8]> for BString<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> Serialize for BString<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match from_utf8(self.0) {
            Ok(string) => serializer.serialize_str(string),
            Err(_) => {
                let string: String = self.0.iter().map(|byte| format!("{:02x}", byte)).collect();
                serializer.serialize_str(&string)
            }
        }
    }
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
    pub fn parse(mut encoded: &'a [u8]) -> Self {
        let window = &mut encoded;
        Self::decode(window)
    }

    fn decode(window: &mut &'a [u8]) -> BValue<'a> {
        match peek(*window) {
            b'd' => {
                advance(window);
                let mut map = HashMap::new();
                while peek(window) != b'e' {
                    let Self::String(key) = Self::decode(window) else {
                        panic!("dictionary keys must be strings")
                    };
                    map.insert(key, Self::decode(window));
                }
                advance(window);
                BValue::Dict(map)
            }
            b'i' => {
                advance(window);
                let integer = parse_int(read_until(window, b'e'));
                BValue::Integer(integer)
            }
            b'l' => {
                advance(window);
                let mut list = vec![];
                while peek(window) != b'e' {
                    list.push(Self::decode(window));
                }
                advance(window);
                BValue::List(list)
            }
            c if c.is_ascii_digit() => {
                let len = parse_int(read_until(window, b':')) as usize;
                BValue::String(read_range(window, len).into())
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
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_string_multi_digit_len() {
        let val = "watermelon";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_string_utf8() {
        let val = "naïve";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        assert_eq!(read(&mut &encoded[..]), b'6');
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_positive() {
        let val = 1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_negative() {
        let val = -1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_integer_zero() {
        let val = 0;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list() {
        let val = ["spam", "eggs"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list_emtpy() {
        let val: [&str; 0] = [];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_list_le_el() {
        let val = ["le", "el"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_dict() {
        let mut val = HashMap::new();
        val.insert("spam", vec!['a', 'b']);
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(json!(decoded), json!(val));
    }

    #[test]
    fn test_read_torrent_file() {
        let encoded = fs::read("sample.torrent").unwrap();
        let decoded = serde_json::to_string_pretty(&BValue::parse(&encoded)).unwrap();
        println!("{decoded}");
    }
}
