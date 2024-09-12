use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::from_utf8_unchecked,
};

use reqwest::blocking::{Client, Request};

use crate::{bencoding::Decoder, bytes_reader::BytesReader};

pub struct QueryParams<'a> {
    pub info_hash: &'a [u8; 20],
    pub peer_id: &'a str,
    pub port: i64,
    pub uploaded: i64,
    pub downloaded: i64,
    pub left: i64,
    pub compact: u8,
}

fn build_request(client: &Client, tracker_url: &str, params: QueryParams) -> Request {
    let info_hash_str = unsafe { from_utf8_unchecked(params.info_hash) };

    client
        .get(tracker_url)
        .query(&[("info_hash", info_hash_str)])
        .query(&[("peer_id", params.peer_id)])
        .query(&[("port", params.port)])
        .query(&[("uploaded", params.uploaded)])
        .query(&[("downloaded", params.downloaded)])
        .query(&[("left", params.left)])
        .query(&[("compact", params.compact)])
        .build()
        .unwrap()
}

pub fn get_peers(tracker_url: &str, query_params: QueryParams) -> Vec<SocketAddrV4> {
    let client = Client::new();
    let request = build_request(&client, tracker_url, query_params);
    let bytes = client.execute(request).unwrap().bytes().unwrap();
    let mut decoder = Decoder::new(BytesReader::new(&bytes));
    decoder.start_dict();
    decoder.find_key("peers");
    decoder
        .read_string_bytes()
        .chunks(6)
        .map(|peer| to_socket_addr(peer))
        .collect()
}

fn to_socket_addr(bytes: &[u8]) -> SocketAddrV4 {
    let ip: [u8; 4] = bytes[..4].try_into().unwrap();
    let port = u16::from_be_bytes(bytes[4..].try_into().unwrap());
    SocketAddrV4::new(Ipv4Addr::from(ip), port)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use reqwest::blocking::Client;

    use crate::{metainfo::Metainfo, tracker::QueryParams};

    use super::{build_request, get_peers};

    fn url_encode(bytes: &[u8]) -> String {
        let mut encoded = String::with_capacity(2 * bytes.len());
        for byte in bytes {
            if !(byte.is_ascii_alphanumeric()
                || *byte == b'-'
                || *byte == b'_'
                || *byte == b'.'
                || *byte == b'~')
            {
                encoded.push('%');
                encoded.push_str(&format!("{:02x}", byte));
            } else {
                encoded.push(*byte as char);
            }
        }
        encoded.shrink_to_fit();
        encoded
    }

    #[test]
    fn test_url_encode() {
        let bytes = hex::decode("d69f91e6b2ae4c542468d1073a71d4ea13879a7f").unwrap();
        let got = url_encode(&bytes);
        let want = "%d6%9f%91%e6%b2%aeLT%24h%d1%07%3aq%d4%ea%13%87%9a%7f";
        assert_eq!(got, want);
    }

    #[test]
    fn test_build_request() {
        let client = Client::new();

        let tracker_url = "http://bittorrent-test-tracker.codecrafters.io/";
        let info_hash_hex = "d69f91e6b2ae4c542468d1073a71d4ea13879a7f";
        let info_hash = &hex::decode(info_hash_hex).unwrap().try_into().unwrap();
        let params = QueryParams {
            info_hash,
            peer_id: "00112233445566778899",
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: 92063,
            compact: 1,
        };

        let request = build_request(&client, tracker_url, params);
        let got = request.url().as_str();
        let mut want = tracker_url.to_owned();
        want += "?info_hash=";
        want += &url_encode(info_hash);
        want += "&peer_id=00112233445566778899";
        want += "&port=6881";
        want += "&uploaded=0";
        want += "&downloaded=0";
        want += "&left=92063";
        want += "&compact=1";
        assert_eq!(got.to_lowercase(), want.to_lowercase());
    }

    #[test]
    fn test_get_peers() {
        let metainfo_path = "sample.torrent";
        let bytes = fs::read(metainfo_path).unwrap();
        let metainfo = Metainfo::from_bytes(&bytes);
        println!("{}", metainfo.announce);
        println!("{}", hex::encode(metainfo.get_info_hash()));

        let info_hash = metainfo.get_info_hash();
        dbg!(info_hash);

        let query_params = QueryParams {
            info_hash: &metainfo.get_info_hash(),
            peer_id: "00112233445566778899",
            port: 6881,
            uploaded: 0,
            downloaded: 0,
            left: metainfo.info.length,
            compact: 1,
        };

        let peers = get_peers(metainfo.announce, query_params);
        assert!(peers.len() != 0);
    }
}
