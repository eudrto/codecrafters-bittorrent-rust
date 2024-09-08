use std::str::from_utf8;

use serde::{Serialize, Serializer};

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
