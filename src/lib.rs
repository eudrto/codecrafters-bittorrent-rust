mod bencoding;
mod bytes_reader;
mod cli;
mod downloader;
mod metainfo;
mod tracker;

use std::fs::{self, read, write};

use clap::Parser;
use tokio::runtime::Runtime;

use bencoding::to_json;
use cli::{Cli, SCommand};
use downloader::{download, peer::Peer};
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

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let my_peer_id = "00112233445566778899";
                let mut peer = Peer::create(&peer_addr.parse().unwrap()).await;
                let peer_id = peer
                    .do_handshake(&metainfo.get_info_hash(), my_peer_id)
                    .await;
                println!("Peer ID: {}", hex::encode(&peer_id))
            })
        }
        SCommand::DownloadPiece {
            output_file_path,
            torrent_file_path,
            piece_no,
        } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

            let pieces = metainfo.into_pieces();
            download(&output_file_path, &metainfo, pieces);

            let piece_no: usize = piece_no.parse().unwrap();
            let contents = read(&output_file_path).unwrap();
            let start = metainfo.get_piece_start(piece_no as u32) as usize;
            let end = start + metainfo.get_piece_len(piece_no as u32) as usize;
            write(output_file_path, &contents[start..end]).unwrap();
        }
        SCommand::Download {
            output_file_path,
            torrent_file_path,
        } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

            let pieces = metainfo.into_pieces();
            download(&output_file_path, &metainfo, pieces);
        }
    }
}
