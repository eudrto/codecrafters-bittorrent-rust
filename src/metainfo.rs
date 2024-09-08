use crate::bencoding::{BDict, BInteger, BString, BValue};

pub struct Metainfo<'a> {
    bvalue: BValue<'a>,
}

impl<'a> Metainfo<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bvalue: BValue::decode(bytes),
        }
    }

    pub fn get_tracker(&self) -> &str {
        let metainfo: &BDict = (&self.bvalue).try_into().unwrap();
        let announce: &BString = (&metainfo["announce"]).try_into().unwrap();
        announce.try_into().unwrap()
    }

    pub fn get_length(&self) -> i64 {
        let metainfo: &BDict = (&self.bvalue).try_into().unwrap();
        let info: &BDict = (&metainfo["info"]).try_into().unwrap();
        let length: &BInteger = (&info["length"]).try_into().unwrap();
        length.as_i64()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Metainfo;

    #[test]
    fn t1() {
        let metainfo_path = "sample.torrent";
        let metainfo_bytes = fs::read(metainfo_path).unwrap();
        let metainfo = Metainfo::new(&metainfo_bytes);
        let tracker = metainfo.get_tracker();
        assert_eq!(
            tracker,
            "http://bittorrent-test-tracker.codecrafters.io/announce"
        );
        let length = metainfo.get_length();
        assert_eq!(length, 92063);
    }
}
