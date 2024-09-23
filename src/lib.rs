mod bencoding;
mod bytes_reader;
mod cli;
mod metainfo;
mod parts;
mod peer;
mod peer_msg;
mod piece_validator;
mod tracker;

use std::fs::{self, write};

use clap::Parser;
use parts::{Piece, PieceReq};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    runtime::Runtime,
    spawn,
    sync::mpsc::{channel, unbounded_channel},
};

use bencoding::to_json;
use cli::{Cli, SCommand};
use metainfo::Metainfo;
use peer::Peer;
use piece_validator::piece_validator;
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

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let mut stream = TcpStream::connect(peer_addr).await.unwrap();
                stream.write(&handshake).await.unwrap();
                stream.read(&mut handshake).await.unwrap();
                println!(
                    "Peer ID: {}",
                    hex::encode(&handshake[handshake.len() - 20..])
                )
            })
        }
        SCommand::DownloadPiece {
            output_file_path,
            torrent_file_path,
            piece_no,
        } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

            let piece_no = piece_no.parse().unwrap();
            let piece_len = metainfo.get_piece_len(piece_no);
            if piece_len == 0 {
                let no_pieces = metainfo.get_no_pieces();
                eprintln!("requested piece: {}, no_pieces: {}", piece_no, no_pieces);
                return;
            }
            let piece_req = PieceReq::new(piece_no, piece_len);

            let query_params = QueryParams {
                info_hash: &metainfo.get_info_hash(),
                peer_id: "00112233445566778899",
                port: 6881,
                uploaded: 0,
                downloaded: 0,
                left: metainfo.info.length,
                compact: 1,
            };
            let peer_addrs = get_peers(metainfo.announce, query_params);

            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let (piece_req_sender, piece_req_receiver) = unbounded_channel();
                piece_req_sender.send(piece_req).unwrap();

                let (block_resp_sender, block_resp_receiver) = channel(1);
                let (piece_resp_sender, mut piece_resp_receiver) = unbounded_channel();

                let peer_id = "00112233445566778899";
                let block_size = 16 * 1024;
                let mut peer = Peer::create(&peer_addrs[0]).await;
                peer.do_handshake(&metainfo.get_info_hash(), peer_id).await;
                peer.init_download().await;
                let (request_writer_task, response_reader_task) =
                    peer.start_download_tasks(piece_req_receiver, block_resp_sender, block_size);

                let validator = spawn(piece_validator(
                    block_resp_receiver,
                    piece_req_sender,
                    piece_resp_sender,
                    Piece::new(
                        piece_no,
                        piece_len,
                        metainfo.info.piece_hashes[piece_no as usize],
                    ),
                ));

                request_writer_task.await.unwrap();
                response_reader_task.await.unwrap();
                validator.await.unwrap();

                let piece_resp = piece_resp_receiver.recv().await.unwrap();
                write(output_file_path, piece_resp.bytes).unwrap();
            });
        }
    }
}
