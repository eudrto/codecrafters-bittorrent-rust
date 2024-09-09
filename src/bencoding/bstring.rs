use std::str::{from_utf8, Utf8Error};

use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct BString<'a>(&'a [u8]);

impl<'a> BString<'a> {
    fn to_hex_string(&self) -> String {
        hex::encode(self.0)
    }
}

impl<'a> From<&'a [u8]> for BString<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> From<&'a str> for BString<'a> {
    fn from(value: &'a str) -> Self {
        Self(value.as_bytes())
    }
}

impl<'a> From<&BString<'a>> for &'a [u8] {
    fn from(value: &BString<'a>) -> Self {
        value.0
    }
}

impl<'a> TryFrom<&BString<'a>> for &'a str {
    type Error = Utf8Error;
    fn try_from(value: &BString<'a>) -> Result<Self, Self::Error> {
        from_utf8(value.0)
    }
}

impl<'a> Serialize for BString<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.try_into() {
            Ok(string) => serializer.serialize_str(string),
            Err(_) => serializer.serialize_str(&self.to_hex_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::BString;

    #[test]
    fn test_serialize_valid_utf8() {
        let value = "apple";
        let bstring = BString::from(value);
        assert_eq!(json!(bstring), value);
    }

    #[test]
    fn test_serialize_invalid_utf8() {
        let value = [0xc0, 0xaf];
        let bstring = BString::from(&value[..]);
        assert_eq!(json!(bstring), "c0af");
    }
}
