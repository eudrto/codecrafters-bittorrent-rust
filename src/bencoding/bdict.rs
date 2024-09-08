use std::{collections::HashMap, ops::Index};

use serde::Serialize;

use super::{BString, BValue};

#[derive(Debug, Serialize)]
pub struct BDict<'a>(HashMap<BString<'a>, BValue<'a>>);

impl<'a> From<HashMap<BString<'a>, BValue<'a>>> for BDict<'a> {
    fn from(value: HashMap<BString<'a>, BValue<'a>>) -> Self {
        Self(value)
    }
}

impl<'a> Index<&'a str> for BDict<'a> {
    type Output = BValue<'a>;
    fn index(&self, index: &'a str) -> &Self::Output {
        self.0.get(&index.into()).unwrap()
    }
}
