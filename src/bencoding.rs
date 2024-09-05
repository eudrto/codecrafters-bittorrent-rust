use serde_json::json;
use std::fmt::Display;

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
}

impl<'a> BValue<'a> {
    pub fn parse(mut encoded: &'a str) -> Self {
        let window = &mut encoded;
        Self::decode(window)
    }

    fn decode(window: &mut &'a str) -> BValue<'a> {
        match peek(*window) {
            'i' => {
                advance(window);
                let integer = read_until(window, 'e').parse::<i64>().unwrap();
                BValue::Integer(integer)
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
}

impl<'a> Display for BValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(string) => {
                write!(f, "{}", json!(string))
            }
            Self::Integer(integer) => {
                write!(f, "{}", json!(integer))
            }
        }
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
}
