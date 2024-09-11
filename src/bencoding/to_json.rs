use std::str::from_utf8;

use crate::bytes_reader::BytesReader;

use super::Decoder;

pub fn to_json(bencoded_value: &[u8]) -> String {
    let mut decoder = Decoder::new(BytesReader::new(bencoded_value));
    let mut json = String::with_capacity(decoder.reader.len());
    decode(&mut decoder, &mut json);
    json.shrink_to_fit();
    json
}

fn decode(decoder: &mut Decoder, json: &mut String) {
    if decoder.reader.is_at_end() {
        return;
    }

    if decoder.is_string() {
        json.push_str("\"");
        let bytes = decoder.read_string_bytes();
        match from_utf8(bytes) {
            Ok(string) => json.push_str(string),
            Err(_) => json.push_str(&hex::encode(bytes)),
        }
        json.push_str("\"");
    } else if decoder.is_integer() {
        json.push_str(&decoder.read_integer().to_string());
    } else if decoder.is_list() {
        decoder.reader.skip();
        json.push_str("[");
        if decoder.reader.peek() != b'e' {
            decode(decoder, json);
        }
        while decoder.reader.peek() != b'e' {
            json.push_str(",");
            decode(decoder, json);
        }
        decoder.reader.skip();
        json.push_str("]");
    } else if decoder.is_dict() {
        decoder.reader.skip();
        json.push_str("{");
        if decoder.reader.peek() != b'e' {
            decode(decoder, json);
            json.push_str(":");
            decode(decoder, json);
        }
        while decoder.reader.peek() != b'e' {
            json.push_str(",");
            decode(decoder, json);
            json.push_str(":");
            decode(decoder, json);
        }
        decoder.reader.skip();
        json.push_str("}");
    } else {
        panic!("invalid encoding")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::{json, Value};

    use super::to_json;

    fn make_round_trip(json: &Value) -> String {
        let bencoded = serde_bencode::to_string(json).unwrap();
        to_json(bencoded.as_bytes())
    }

    #[test]
    fn test_to_json_utf8() {
        let v = json!("naïve");
        assert_eq!(make_round_trip(&v), v.to_string());
    }

    #[test]
    fn test_to_json() {
        let v = json!(
        {
          "a": {
            "b": {
              "c": [
                1,
                2,
                {
                  "d": "text1",
                  "e": [
                    3,
                    {
                      "f": "text2"
                    }
                  ]
                }
              ]
            },
            "g": {
              "h": {
                "i": 4,
                "j": [
                  "text3",
                  {
                    "k": 5
                  }
                ]
              }
            }
          }
        });

        let decoded = make_round_trip(&v);
        assert_eq!(decoded, v.to_string());
    }

    #[test]
    fn test_to_json_sample_torrent() {
        let metainfo_path = "sample.torrent";
        let bytes = fs::read(metainfo_path).unwrap();
        to_json(&bytes);
    }
}
