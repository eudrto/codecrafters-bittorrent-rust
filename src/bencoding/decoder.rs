use std::str::from_utf8;

use crate::bytes_reader::BytesReader;

#[derive(Debug)]
pub struct Decoder<'a> {
    reader: BytesReader<'a>,
}

impl<'a> Decoder<'a> {
    pub fn new(reader: BytesReader<'a>) -> Self {
        Self { reader }
    }

    fn is_string(&self) -> bool {
        self.reader.peek().is_ascii_digit()
    }

    fn is_integer(&self) -> bool {
        self.reader.peek() == b'i'
    }

    fn is_list(&self) -> bool {
        self.reader.peek() == b'l'
    }

    fn is_dict(&self) -> bool {
        self.reader.peek() == b'd'
    }

    pub fn read_string_bytes(&mut self) -> &'a [u8] {
        let len = from_utf8(self.reader.read_until(b':'))
            .unwrap()
            .parse::<usize>()
            .unwrap();
        self.reader.skip();
        self.reader.read_n(len)
    }

    pub fn read_string(&mut self) -> &'a str {
        from_utf8(self.read_string_bytes()).unwrap()
    }

    pub fn read_integer_bytes(&mut self) -> &'a [u8] {
        self.reader.skip();
        let integer = self.reader.read_until(b'e');
        self.reader.skip();
        integer
    }

    pub fn read_integer(&mut self) -> i64 {
        from_utf8(self.read_integer_bytes())
            .unwrap()
            .parse::<i64>()
            .unwrap()
    }

    pub fn start_dict(&mut self) -> usize {
        if !self.is_dict() {
            panic!("not a dict")
        }
        self.reader.skip();
        self.reader.get_pos() - 1
    }

    pub fn find_key(&mut self, needle: &str) {
        while self.reader.peek() != b'e' {
            let key = self.read_string();
            if key == needle {
                return;
            }
            self.parse();
        }
        panic!("{} not found", needle);
    }

    pub fn finish_dict(&mut self, start: usize) -> &'a [u8] {
        while self.reader.peek() != b'e' {
            self.parse();
            self.parse();
        }
        self.reader.skip();
        self.reader.get_from(start)
    }

    fn parse(&mut self) {
        if self.reader.is_at_end() {
            return;
        }

        if self.is_string() {
            self.read_string_bytes();
        } else if self.is_integer() {
            self.read_integer_bytes();
        } else if self.is_list() {
            self.reader.skip();
            while self.reader.peek() != b'e' {
                self.parse();
            }
            self.reader.skip();
        } else if self.is_dict() {
            self.reader.skip();
            while self.reader.peek() != b'e' {
                self.parse();
                self.parse();
            }
            self.reader.skip();
        } else {
            panic!("invalid encoding")
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Decoder;
    use crate::bytes_reader::BytesReader;

    fn get_test_value() -> serde_json::Value {
        json!({
            "a": 1,
            "b": {
                "c": 2,
                "d": {
                    "e": 3,
                    "f": 4
                },
                "g": 5
            },
            "h": 6
        })
    }

    #[test]
    fn test_decoder_read_all() {
        let value = get_test_value();
        let encoded = serde_bencode::to_string(&value).unwrap();

        let bytes_reader = BytesReader::new(encoded.as_bytes());
        let mut decoder = Decoder::new(bytes_reader);

        // outer -->
        let root_start = decoder.start_dict();
        decoder.find_key("a");
        assert_eq!(decoder.read_integer(), 1);
        decoder.find_key("b");

        // middle -->
        let b_start = decoder.start_dict();
        decoder.find_key("c");
        assert_eq!(decoder.read_integer(), 2);
        decoder.find_key("d");

        // inner -->
        let d_start = decoder.start_dict();
        decoder.find_key("e");
        assert_eq!(decoder.read_integer(), 3);
        decoder.find_key("f");
        assert_eq!(decoder.read_integer(), 4);

        let d = decoder.finish_dict(d_start);
        let d_want = json!({
            "e": 3,
            "f": 4
        });
        assert_eq!(d, serde_bencode::to_bytes(&d_want).unwrap());
        // <-- inner

        decoder.find_key("g");
        assert_eq!(decoder.read_integer(), 5);

        let b = decoder.finish_dict(b_start);
        let b_want = json!({
            "c": 2,
            "d": {
                "e": 3,
                "f": 4
            },
            "g": 5
        });
        assert_eq!(b, serde_bencode::to_bytes(&b_want).unwrap());
        // <-- middle

        decoder.find_key("h");
        assert_eq!(decoder.read_integer(), 6);

        let root = decoder.finish_dict(root_start);
        assert_eq!(root, encoded.as_bytes());
        // <-- outer
    }

    #[test]
    fn test_decoder_start_end() {
        let value = get_test_value();
        let encoded = serde_bencode::to_string(&value).unwrap();

        let bytes_reader = BytesReader::new(encoded.as_bytes());
        let mut decoder = Decoder::new(bytes_reader);

        // outer -->
        let root_start = decoder.start_dict();
        decoder.find_key("b");

        // middle -->
        let b_start = decoder.start_dict();
        decoder.find_key("d");

        // inner -->
        let d_start = decoder.start_dict();

        let d = decoder.finish_dict(d_start);
        let d_want = json!({
            "e": 3,
            "f": 4
        });
        assert_eq!(d, serde_bencode::to_bytes(&d_want).unwrap());
        // <-- inner

        let b = decoder.finish_dict(b_start);
        let b_want = json!({
            "c": 2,
            "d": {
                "e": 3,
                "f": 4
            },
            "g": 5
        });
        assert_eq!(b, serde_bencode::to_bytes(&b_want).unwrap());
        // <-- middle

        let root = decoder.finish_dict(root_start);
        assert_eq!(root, encoded.as_bytes());
        // <-- outer
    }

    #[test]
    fn test_decoder_skip_inner() {
        let value = get_test_value();
        let encoded = serde_bencode::to_string(&value).unwrap();

        let bytes_reader = BytesReader::new(encoded.as_bytes());
        let mut decoder = Decoder::new(bytes_reader);

        // outer -->
        let root_start = decoder.start_dict();
        decoder.find_key("a");
        assert_eq!(decoder.read_integer(), 1);
        decoder.find_key("b");

        // middle -->
        let b_start = decoder.start_dict();
        decoder.find_key("c");
        assert_eq!(decoder.read_integer(), 2);
        decoder.find_key("g");
        assert_eq!(decoder.read_integer(), 5);

        let b = decoder.finish_dict(b_start);
        let b_want = json!({
            "c": 2,
            "d": {
                "e": 3,
                "f": 4
            },
            "g": 5
        });
        assert_eq!(b, serde_bencode::to_bytes(&b_want).unwrap());
        // <-- middle

        decoder.find_key("h");
        assert_eq!(decoder.read_integer(), 6);

        let root = decoder.finish_dict(root_start);
        assert_eq!(root, encoded.as_bytes());
        // <-- outer
    }
}
