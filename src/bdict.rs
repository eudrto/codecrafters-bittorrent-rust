use std::collections::HashMap;

use serde::Serialize;

use crate::{bencoding::BValue, bstring::BString};

#[derive(Serialize)]
pub struct BDict<'a>(HashMap<BString<'a>, BValue<'a>>);

impl<'a> From<HashMap<BString<'a>, BValue<'a>>> for BDict<'a> {
    fn from(value: HashMap<BString<'a>, BValue<'a>>) -> Self {
        Self(value)
    }
}
