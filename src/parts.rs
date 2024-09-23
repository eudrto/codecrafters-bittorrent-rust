use crate::peer_msg::PeerMsg;

#[derive(Debug)]
pub struct BlockReq {
    piece_idx: u32,
    begin: u32,
    len: u32,
}

impl BlockReq {
    pub fn new(piece_idx: u32, begin: u32, len: u32) -> Self {
        Self {
            piece_idx,
            begin,
            len,
        }
    }
}

impl From<BlockReq> for PeerMsg {
    fn from(block_req: BlockReq) -> Self {
        Self::Request {
            idx: block_req.piece_idx,
            begin: block_req.begin,
            length: block_req.len,
        }
    }
}

pub struct BlockResp {
    pub begin: u32,
    pub bytes: Vec<u8>,
}

impl BlockResp {
    pub fn new(begin: u32, bytes: Vec<u8>) -> Self {
        Self { begin, bytes }
    }
}

#[derive(Debug)]
pub struct Piece {
    pub idx: u32,
    pub len: u32,
    pub hash: [u8; 20],
}

impl Piece {
    pub fn new(idx: u32, len: u32, hash: [u8; 20]) -> Self {
        Self { idx, len, hash }
    }
}

impl From<&Piece> for PieceReq {
    fn from(piece: &Piece) -> Self {
        Self {
            idx: piece.idx,
            len: piece.len,
        }
    }
}

#[derive(Debug)]
pub struct PieceReq {
    pub idx: u32,
    pub len: u32,
}

impl PieceReq {
    pub fn new(idx: u32, len: u32) -> Self {
        Self { idx, len }
    }

    pub fn into_block_reqs(&self, block_size: u32) -> Vec<BlockReq> {
        let mut blocks: Vec<_> = (0..self.len)
            .step_by(block_size as usize)
            .map(|begin| BlockReq::new(self.idx, begin, block_size))
            .collect();
        let no_blocks = blocks.len();
        blocks[no_blocks - 1].len = self.len - (no_blocks - 1) as u32 * block_size;
        blocks
    }
}

pub struct PieceResp {
    pub _idx: u32,
    pub bytes: Vec<u8>,
}

impl PieceResp {
    pub fn from_piece(piece: Piece, bytes: Vec<u8>) -> Self {
        Self {
            _idx: piece.idx,
            bytes,
        }
    }
}
