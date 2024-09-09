use std::{collections::HashMap, ops::Index};

use serde::Serialize;

use super::{BString, BValue};

#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct BDict<'a> {
    #[serde(skip_serializing)]
    encoded: &'a [u8],
    map: HashMap<BString<'a>, BValue<'a>>,
}

impl<'a> BDict<'a> {
    pub fn new(encoded: &'a [u8], map: HashMap<BString<'a>, BValue<'a>>) -> Self {
        Self { encoded, map }
    }

    pub fn encode(&self) -> &'a [u8] {
        self.encoded
    }
}

impl<'a> Index<&'a str> for BDict<'a> {
    type Output = BValue<'a>;
    fn index(&self, index: &'a str) -> &Self::Output {
        self.map.get(&index.into()).unwrap()
    }
}
