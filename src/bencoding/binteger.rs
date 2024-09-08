use std::str::from_utf8;

use serde::{Serialize, Serializer};

#[derive(Debug)]
pub struct BInteger<'a>(&'a [u8]);

impl<'a> BInteger<'a> {
    pub fn as_i64(&self) -> i64 {
        from_utf8(self.0).unwrap().parse::<i64>().unwrap()
    }
}

impl<'a> From<&'a [u8]> for BInteger<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self(value)
    }
}

impl<'a> Serialize for BInteger<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.as_i64())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::BInteger;

    #[test]
    fn test_serialize() {
        let value = [b'4', b'2'];
        let bstring = BInteger::from(&value[..]);
        assert_eq!(json!(bstring), 42);
    }
}
