mod bencoding;
mod bytes_reader;
mod cli;
mod metainfo;
mod peer_msg;
mod tracker;

use std::{
    cmp::min,
    fs::{self, write},
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
};

use bencoding::to_json;
use clap::Parser;
use cli::{Cli, SCommand};
use metainfo::Metainfo;
use peer_msg::PeerMsg;
use sha1::{Digest, Sha1};
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
        SCommand::DownloadPiece {
            output_file_path,
            torrent_file_path,
            piece_no: piece_want,
        } => {
            let bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::from_bytes(&bytes);

            let piece_want: usize = piece_want.parse().unwrap();
            let no_pieces = metainfo.info.piece_hashes.len();
            if piece_want >= no_pieces {
                eprint!("requested piece: {}, max: {}", piece_want, no_pieces);
                return;
            }

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

            let stream = TcpStream::connect(peers[0]).unwrap();
            let mut reader = BufReader::new(&stream);
            let mut writer = BufWriter::new(&stream);

            // handshake
            let mut handshake: [u8; 68] = [0; 68];
            let proto = "BitTorrent protocol";
            handshake[0] = proto.len() as u8;
            handshake[1..20].copy_from_slice(proto.as_bytes());
            handshake[28..48].copy_from_slice(&metainfo.get_info_hash());
            handshake[48..68].copy_from_slice("00112233445566778899".as_bytes());
            writer.write_all(&handshake).unwrap();
            writer.flush().unwrap();

            reader.read_exact(&mut handshake).unwrap();

            // <-- bitfield
            loop {
                let msg = PeerMsg::read(&mut reader);
                if let PeerMsg::Bitfield(_) = msg {
                    break;
                }
                dbg!(msg);
            }

            // interested -->
            PeerMsg::Interested.write(&mut writer);

            // <-- unchoke
            loop {
                let msg = PeerMsg::read(&mut reader);
                if let PeerMsg::Unchoke = msg {
                    break;
                }
                dbg!(msg);
            }

            // requests
            const BLOCK_SIZE: usize = 16 * 1024;

            let piece_length = min(
                metainfo.info.piece_length,
                metainfo.info.length - piece_want as i64 * metainfo.info.piece_length,
            );

            let mut piece = vec![0; piece_length as usize];
            let blocks = piece.chunks_mut(BLOCK_SIZE);

            for (block_idx, block) in blocks.into_iter().enumerate() {
                let req = PeerMsg::Request {
                    idx: piece_want as u32,
                    begin: (block_idx * BLOCK_SIZE) as u32,
                    length: block.len() as u32,
                };
                req.write(&mut writer);

                let (piece_idx, block_begin) = loop {
                    let msg = PeerMsg::read(&mut reader);
                    if let PeerMsg::Piece { idx, begin } = msg {
                        break (idx, begin);
                    }
                };
                assert_eq!(piece_idx, piece_want as u32);
                assert_eq!(block_begin as usize % BLOCK_SIZE, 0);

                reader.read_exact(block).unwrap();
            }

            // piece hash
            let mut hasher = Sha1::new();
            hasher.update(&piece);
            let hash: [u8; 20] = hasher.finalize().into();
            assert_eq!(hash, metainfo.info.piece_hashes[piece_want]);

            // write
            write(output_file_path, piece).unwrap();
        }
    }
}
