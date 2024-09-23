//  Concurrent tasks:
//
//  -------------piece-req-------------
//  |                                 |
//  |                    ---------------------------
//  |                    |                         |
//  |                    V                         V
//  |           |-----------------|       |-----------------|
//  |           |      Peer       |       |      Peer       |
//  |           |-----------------|       |-----------------|
//  |           | request_writer  |       | request_writer  |
//  |           |        |        |       |        |        |
//  |           |        V        |       |        V        |
//  |           | response_reader |       | response_reader |
//  |           |-----------------|       |-----------------|
//  |               a    b    c               a    b    c
//  |               |    |    |               |    |    |
//  |            ---+----+----+---block-resp---    |    |
//  |            |       |    |                    |    |
//  |            |       -----+------+--block-resp--    |
//  |            a            |      |                  |
//  |            V            -------+------block-resp---
//  |   |--------|--------|          |                  |
//  |---| piece_validator |          |                  |
//  |   |-----------------|          b                  |
//  |            x                   V                  |
//  |            |          |--------|--------|         |
//  |------------+----------| piece_validator |         |
//  |            |          |-----------------|         c
//  |            |                   x                  V
//  |            |                   |          |-----------------|
//  -------------+-------------------+----------| piece_validator |
//               |                   |          |-----------------|
//               |                   |                  x
//               |                                      |
//               ---------------piece-resp---------------
//                                   |
//                                   x
//                                   V
//                              |----------|
//                              | combiner |
//                              |----------|

pub mod parts;
pub mod peer;
mod peer_msg;
mod piece_combiner;
mod piece_validator;

use async_channel::unbounded;
use tokio::{
    runtime::Runtime,
    spawn,
    sync::mpsc::{channel, unbounded_channel},
};

use crate::{
    metainfo::Metainfo,
    tracker::{get_peers, QueryParams},
};
use parts::Piece;
use peer::Peer;
use piece_combiner::piece_combiner;
use piece_validator::piece_validator;

pub fn download(output_file_path: &str, metainfo: &Metainfo, pieces: Vec<Piece>) {
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
        let (piece_req_sender, piece_req_receiver) = unbounded();
        for piece in &pieces {
            piece_req_sender.send(piece.into()).await.unwrap();
        }

        let (block_resp_senders, block_resp_receivers): (Vec<_>, Vec<_>) =
            (0..pieces.len()).map(|_| channel(1)).unzip();

        let (piece_resp_sender, piece_resp_receiver) = unbounded_channel();

        let mut request_writer_tasks = vec![];
        let mut response_reader_tasks = vec![];

        let peer_id = "00112233445566778899";
        let block_size = 16 * 1024;
        for addr in &peer_addrs {
            let mut peer = Peer::create(addr).await;
            peer.do_handshake(&metainfo.get_info_hash(), peer_id).await;
            peer.init_download().await;
            let (request_writer_task, response_reader_task) = peer.start_download_tasks(
                piece_req_receiver.clone(),
                block_resp_senders.clone(),
                block_size,
            );
            request_writer_tasks.push(request_writer_task);
            response_reader_tasks.push(response_reader_task);
        }

        let mut validator_tasks = vec![];
        for (block_receiver, piece) in block_resp_receivers.into_iter().zip(pieces) {
            let task = spawn(piece_validator(
                block_receiver,
                piece_req_sender.clone(),
                piece_resp_sender.clone(),
                piece,
            ));
            validator_tasks.push(task);
        }

        let combiner_task = spawn(piece_combiner(
            piece_resp_receiver,
            metainfo.info.piece_length,
            String::from(output_file_path),
        ));

        drop(piece_req_sender);
        drop(piece_resp_sender);

        for task in request_writer_tasks {
            task.await.unwrap();
        }
        for task in response_reader_tasks {
            task.await.unwrap();
        }
        for validator in validator_tasks {
            validator.await.unwrap();
        }
        combiner_task.await.unwrap();
    });
}
