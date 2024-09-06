use serde_json::{json, Map, Number, Value};
use std::{collections::HashMap, fmt::Display, str::from_utf8};

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

pub enum BValue<'a> {
    String(&'a [u8]),
    Integer(i64),
    List(Vec<BValue<'a>>),
    Dict(HashMap<&'a [u8], BValue<'a>>),
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
                BValue::String(read_range(window, len))
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::String(bytes) => match from_utf8(bytes) {
                Ok(string) => Value::String(string.to_string()),
                Err(_) => {
                    let string = bytes.iter().map(|byte| format!("{:02x}", byte)).collect();
                    Value::String(string)
                }
            },
            Self::Integer(integer) => Value::Number(Number::from(*integer)),
            Self::List(list) => Value::Array(list.iter().map(|bval| bval.to_value()).collect()),
            Self::Dict(dict) => Value::Object(Map::from_iter(
                dict.iter()
                    .map(|(k, v)| (from_utf8(k).unwrap().to_string(), v.to_value()))
                    .collect::<Vec<_>>(),
            )),
        }
    }
}

impl<'a> Display for BValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", json!(self.to_value()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use serde_bencode;

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
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_string_multi_digit_len() {
        let val = "watermelon";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_string_utf8() {
        let val = "naïve";
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        assert_eq!(read(&mut &encoded[..]), b'6');
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_positive() {
        let val = 1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_negative() {
        let val = -1;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_zero() {
        let val = 0;
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list() {
        let val = ["spam", "eggs"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list_emtpy() {
        let val: [&str; 0] = [];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list_le_el() {
        let val = ["le", "el"];
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_dict() {
        let mut val = HashMap::new();
        val.insert("spam", vec!['a', 'b']);
        let encoded = serde_bencode::to_bytes(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_read_torrent_file() {
        let encoded = fs::read("sample.torrent").unwrap();
        BValue::parse(&encoded);
    }
}
