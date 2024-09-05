use serde_json::{json, Map, Number, Value};
use std::{collections::HashMap, fmt::Display};

fn peek(bytes: &str) -> char {
    bytes.chars().next().unwrap()
}

fn advance(window: &mut &str) {
    read(window);
}

fn read(window: &mut &str) -> char {
    let result = peek(window);
    *window = &window[result.len_utf8()..];
    result
}

fn read_range<'a>(window: &mut &'a str, len: usize) -> &'a str {
    let result = &window[..len];
    *window = &window[len..];
    result
}

fn read_until<'a>(window: &mut &'a str, ch: char) -> &'a str {
    let colon_index = window.find(ch).unwrap();
    let result = &window[..colon_index];
    *window = &window[colon_index + 1..];
    result
}

pub enum BValue<'a> {
    String(&'a str),
    Integer(i64),
    List(Vec<BValue<'a>>),
    Dict(HashMap<&'a str, BValue<'a>>),
}

impl<'a> BValue<'a> {
    pub fn parse(mut encoded: &'a str) -> Self {
        let window = &mut encoded;
        Self::decode(window)
    }

    fn decode(window: &mut &'a str) -> BValue<'a> {
        match peek(*window) {
            'd' => {
                advance(window);
                let mut map = HashMap::new();
                while peek(window) != 'e' {
                    let Self::String(key) = Self::decode(window) else {
                        panic!("dictionary keys must be strings")
                    };
                    map.insert(key, Self::decode(window));
                }
                advance(window);
                BValue::Dict(map)
            }
            'i' => {
                advance(window);
                let integer = read_until(window, 'e').parse::<i64>().unwrap();
                BValue::Integer(integer)
            }
            'l' => {
                advance(window);
                let mut list = vec![];
                while peek(window) != 'e' {
                    list.push(Self::decode(window));
                }
                advance(window);
                BValue::List(list)
            }
            c if c.is_ascii_digit() => {
                let len = read_until(window, ':').parse::<usize>().unwrap();
                BValue::String(read_range(window, len))
            }
            _ => {
                unimplemented!()
            }
        }
    }

    fn to_value(&self) -> Value {
        match self {
            Self::String(string) => Value::String(string.to_string()),
            Self::Integer(integer) => Value::Number(Number::from(*integer)),
            Self::List(list) => Value::Array(list.iter().map(|bval| bval.to_value()).collect()),
            Self::Dict(dict) => Value::Object(Map::from_iter(
                dict.iter()
                    .map(|(k, v)| (k.to_string(), v.to_value()))
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
    use super::*;
    use serde_bencode;

    #[test]
    fn test_read() {
        let mut val = "Äpfel";
        let window = &mut val;
        let char = read(window);
        assert_eq!(char, 'Ä');
        assert_eq!(*window, "pfel");
    }

    #[test]
    fn test_string_single_digit_len() {
        let val = "apple";
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_string_multi_digit_len() {
        let val = "watermelon";
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_string_utf8() {
        let val = "naïve";
        let encoded = serde_bencode::to_string(&val).unwrap();
        assert_eq!(read(&mut &encoded[..]), '6');
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_positive() {
        let val = 1;
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_negative() {
        let val = -1;
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_integer_zero() {
        let val = 0;
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list() {
        let val = ["spam", "eggs"];
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list_emtpy() {
        let val: [&str; 0] = [];
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_list_le_el() {
        let val = ["le", "el"];
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }

    #[test]
    fn test_dict() {
        let mut val = HashMap::new();
        val.insert("spam", vec!['a', 'b']);
        let encoded = serde_bencode::to_string(&val).unwrap();
        let decoded = BValue::parse(&encoded);
        assert_eq!(decoded.to_string(), json!(val).to_string());
    }
}
