use async_channel::Sender;
use sha1::{Digest, Sha1};
use tokio::sync::mpsc::{Receiver, UnboundedSender};

use super::parts::{BlockResp, Piece, PieceReq, PieceResp};

pub async fn piece_validator(
    mut block_resp_receiver: Receiver<BlockResp>,
    piece_req_sender: Sender<PieceReq>,
    piece_resp_sender: UnboundedSender<PieceResp>,
    piece: Piece,
) {
    loop {
        let mut blocks = vec![];
        loop {
            let Some(block) = block_resp_receiver.recv().await else {
                return;
            };
            blocks.push(block);

            match check_completeness(piece.len, &mut blocks) {
                State::Incomplete => {
                    continue;
                }
                State::Complete => {
                    let bytes: Vec<_> = blocks.into_iter().flat_map(|block| block.bytes).collect();
                    if !is_valid(&piece.hash, &bytes) {
                        break;
                    }
                    let piece_resp = PieceResp::from_piece(piece, bytes);
                    piece_resp_sender.send(piece_resp).unwrap();
                    return;
                }
                State::Invalid => break,
            };
        }
        drain(&mut block_resp_receiver);
        piece_req_sender.send((&piece).into()).await.unwrap();
    }
}

enum State {
    Incomplete,
    Complete,
    Invalid,
}

fn check_completeness(piece_len: u32, blocks: &mut Vec<BlockResp>) -> State {
    blocks.sort_by_key(|block| block.begin);

    let mut begin = 0;
    for block in blocks.iter() {
        if block.begin > begin {
            return State::Incomplete;
        }
        if block.begin < begin {
            // overlap
            return State::Invalid;
        }
        begin += block.bytes.len() as u32;
    }
    if begin < piece_len {
        return State::Incomplete;
    }
    if begin > piece_len {
        // too long
        return State::Invalid;
    }

    State::Complete
}

fn is_valid(piece_hash: &[u8; 20], bytes: &[u8]) -> bool {
    let mut hasher = Sha1::new();
    hasher.update(&bytes);
    let hash: [u8; 20] = hasher.finalize().into();
    hash == *piece_hash
}

fn drain(receiver: &mut Receiver<BlockResp>) {
    loop {
        if let Err(_) = receiver.try_recv() {
            return;
        }
    }
}
