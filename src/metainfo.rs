use std::{
    cmp::{max, min},
    fmt::{self, Display},
};

use sha1::{Digest, Sha1};

use crate::{bencoding::Decoder, bytes_reader::BytesReader, downloader::parts::Piece};

pub struct Info<'a> {
    pub encoded: &'a [u8],
    pub length: u64,
    pub piece_length: u32,
    pub piece_hashes: Vec<[u8; 20]>,
}

impl<'a> Info<'a> {
    pub fn decode(decoder: &mut Decoder<'a>) -> Self {
        let start = decoder.start_dict();

        decoder.find_key("length");
        let length = decoder.read_integer();

        decoder.find_key("piece length");
        let piece_length = decoder.read_integer();

        decoder.find_key("pieces");
        let pieces = decoder.read_string_bytes();
        let piece_hashes = pieces
            .chunks(20)
            .map(|hash| hash.try_into().unwrap())
            .collect();

        let encoded = decoder.finish_dict(start);

        Self {
            encoded,
            length: length as u64,
            piece_length: piece_length as u32,
            piece_hashes,
        }
    }
}

pub struct Metainfo<'a> {
    pub announce: &'a str,
    pub info: Info<'a>,
}

impl<'a> Metainfo<'a> {
    pub fn from_bytes(bytes: &'a [u8]) -> Self {
        Metainfo::decode(&mut Decoder::new(BytesReader::new(bytes)))
    }

    pub fn decode(decoder: &mut Decoder<'a>) -> Self {
        let start = decoder.start_dict();

        decoder.find_key("announce");
        let announce = decoder.read_string();

        decoder.find_key("info");
        let info = Info::decode(decoder);

        decoder.finish_dict(start);

        Self { announce, info }
    }

    pub fn get_info_hash(&self) -> [u8; 20] {
        let mut hasher = Sha1::new();
        hasher.update(self.info.encoded);
        let hash = hasher.finalize();
        hash.into()
    }

    pub fn get_no_pieces(&self) -> usize {
        self.info.piece_hashes.len()
    }

    pub fn get_piece_start(&self, piece_idx: u32) -> u64 {
        piece_idx as u64 * self.info.piece_length as u64
    }

    pub fn get_piece_len(&self, piece_idx: u32) -> u32 {
        min(
            self.info.piece_length,
            max(
                0,
                (self.info.length - self.get_piece_start(piece_idx)) as u32,
            ),
        )
    }

    fn get_last_piece_len(&self) -> u32 {
        let no_pieces = self.get_no_pieces() as u64;
        let last_piece_len = self.info.length - (no_pieces - 1) * self.info.piece_length as u64;
        last_piece_len as u32
    }

    pub fn into_pieces(&self) -> Vec<Piece> {
        let piece_hashes = &self.info.piece_hashes;
        let mut pieces: Vec<_> = piece_hashes
            .iter()
            .enumerate()
            .map(|(idx, hash)| Piece::new(idx as u32, self.info.piece_length, *hash))
            .collect();
        pieces.last_mut().unwrap().len = self.get_last_piece_len();
        pieces
    }
}

impl<'a> Display for Metainfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tracker URL: {}", self.announce)?;
        write!(f, "Length: {}", self.info.length)?;
        write!(f, "Info Hash: {}", hex::encode(self.get_info_hash()))?;
        write!(f, "Piece Length: {}", self.info.piece_length)?;
        write!(f, "Piece Hashes:")?;
        for hash in &self.info.piece_hashes {
            write!(f, "{}", hex::encode(hash))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::Metainfo;

    #[test]
    fn test_metainfo() {
        let metainfo_path = "sample.torrent";
        let bytes = fs::read(metainfo_path).unwrap();
        let metainfo = Metainfo::from_bytes(&bytes);

        let announce_want = "http://bittorrent-test-tracker.codecrafters.io/announce";
        assert_eq!(metainfo.announce, announce_want);

        assert_eq!(metainfo.info.length, 92063);
        assert_eq!(metainfo.info.piece_length, 32768);

        let piece_hashes_want = vec![
            [
                232, 118, 246, 122, 42, 136, 134, 232, 243, 107, 19, 103, 38, 195, 15, 162, 151, 3,
                2, 45,
            ],
            [
                110, 34, 117, 230, 4, 160, 118, 102, 86, 115, 110, 129, 255, 16, 181, 82, 4, 173,
                141, 53,
            ],
            [
                240, 13, 147, 122, 2, 19, 223, 25, 130, 188, 141, 9, 114, 39, 173, 158, 144, 154,
                204, 23,
            ],
        ];
        assert_eq!(metainfo.info.piece_hashes, piece_hashes_want);
    }
}
