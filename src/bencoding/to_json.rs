use std::str::from_utf8;

use serde_bencode::value::Value as BValue;
use serde_json::Value as JValue;

fn to_string(bytes: &[u8]) -> String {
    match from_utf8(bytes) {
        Ok(string) => string.to_owned(),
        Err(_) => hex::encode(bytes),
    }
}

fn convert(bval: &BValue) -> JValue {
    match bval {
        BValue::Bytes(bytes) => JValue::String(to_string(bytes)),
        BValue::Int(int) => JValue::Number(serde_json::Number::from(*int)),
        BValue::List(list) => JValue::Array(list.iter().map(|v| convert(v)).collect()),
        BValue::Dict(dict) => JValue::Object(
            dict.iter()
                .map(|(k, v)| (to_string(k), convert(v)))
                .collect(),
        ),
    }
}

pub fn to_json(bencoded_value: &str) -> String {
    convert(&serde_bencode::from_str(&bencoded_value).unwrap()).to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use super::to_json;

    fn make_round_trip(json: &Value) -> String {
        let bencoded = serde_bencode::to_string(json).unwrap();
        to_json(&bencoded)
    }

    #[test]
    fn test_to_json_utf8() {
        let v = json!("naïve");
        assert_eq!(make_round_trip(&v), v.to_string());
    }

    #[test]
    fn test_to_json() {
        // See if this json survives a round trip
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
}
