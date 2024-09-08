use serde::Serialize;

use super::BValue;


#[derive(Serialize)]
pub struct BList<'a>(Vec<BValue<'a>>);

impl<'a> From<Vec<BValue<'a>>> for BList<'a> {
    fn from(value: Vec<BValue<'a>>) -> Self {
        Self(value)
    }
}
