use sha1::{Digest, Sha1};

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

    fn get_info(&'a self) -> &BDict<'a> {
        let metainfo: &BDict = (&self.bvalue).try_into().unwrap();
        (&metainfo["info"]).try_into().unwrap()
    }

    pub fn get_info_hash(&self) -> String {
        let encoded = self.get_info().encode();
        let mut hasher = Sha1::new();
        hasher.update(encoded);
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    pub fn get_length(&self) -> i64 {
        let info = self.get_info();
        let length: &BInteger = (&info["length"]).try_into().unwrap();
        length.as_i64()
    }

    pub fn get_piece_length(&self) -> i64 {
        let info = self.get_info();
        let piece_length: &BInteger = (&info["piece length"]).try_into().unwrap();
        piece_length.as_i64()
    }

    pub fn get_piece_hashes(&self) -> Vec<String> {
        let info = self.get_info();
        let piece_hashes: &BString = (&info["pieces"]).try_into().unwrap();
        let bytes: &[u8] = piece_hashes.into();
        bytes.chunks(20).map(|hash| hex::encode(hash)).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Metainfo;

    #[test]
    fn test_metainfo() {
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
