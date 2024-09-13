mod bencoding;
mod bytes_reader;
mod cli;
mod metainfo;
mod tracker;

use std::{
    fs,
    io::{Read, Write},
    net::TcpStream,
};

use bencoding::to_json;
use clap::Parser;
use cli::{Cli, SCommand};
use metainfo::Metainfo;
use tracker::{get_peers, QueryParams};

pub fn run() {
    let cli = Cli::parse();

    match cli.s_command {
        SCommand::Decode { bencoded_value } => {
            println!("{}", to_json(bencoded_value.as_bytes()));
        }
        SCommand::Info { torrent_file_path } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);
            println!("{}", metainfo);
        }
        SCommand::Peers { torrent_file_path } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

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

            for peer in peers {
                println!("{}", peer);
            }
        }
        SCommand::Handshake {
            torrent_file_path,
            peer_addr,
        } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

            let mut handshake: [u8; 68] = [0; 68];
            let proto = "BitTorrent protocol";
            handshake[0] = proto.len() as u8;
            handshake[1..20].copy_from_slice(proto.as_bytes());
            handshake[28..48].copy_from_slice(&metainfo.get_info_hash());
            handshake[48..68].copy_from_slice("00112233445566778899".as_bytes());

            let mut stream = TcpStream::connect(peer_addr).unwrap();
            stream.write(&handshake).unwrap();
            stream.read(&mut handshake).unwrap();
            println!(
                "Peer ID: {}",
                hex::encode(&handshake[handshake.len() - 20..])
            )
        }
    }
}
