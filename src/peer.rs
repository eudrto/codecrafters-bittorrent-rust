use std::net::SocketAddrV4;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    spawn,
    sync::mpsc::{channel, Sender, UnboundedReceiver},
    task::JoinHandle,
};

use crate::{
    peer_msg::PeerMsg,
    parts::{BlockResp, PieceReq},
};

pub struct Peer {
    _addr: SocketAddrV4,
    reader: BufReader<OwnedReadHalf>,
    writer: BufWriter<OwnedWriteHalf>,
}

impl Peer {
    fn new(
        addr: SocketAddrV4,
        reader: BufReader<OwnedReadHalf>,
        writer: BufWriter<OwnedWriteHalf>,
    ) -> Self {
        Self {
            _addr: addr,
            reader,
            writer,
        }
    }

    pub async fn create(addr: &SocketAddrV4) -> Self {
        let stream = TcpStream::connect(addr).await.unwrap();
        let (read_half, write_half) = stream.into_split();
        let reader = BufReader::new(read_half);
        let writer = BufWriter::new(write_half);
        Self::new(*addr, reader, writer)
    }

    pub async fn do_handshake(&mut self, info_hash: &[u8; 20], peer_id: &str) {
        let mut msg: [u8; 68] = [0; 68];
        let proto = "BitTorrent protocol";
        msg[0] = proto.len() as u8;
        msg[1..20].copy_from_slice(proto.as_bytes());
        msg[28..48].copy_from_slice(info_hash);
        msg[48..68].copy_from_slice(peer_id.as_bytes());
        self.writer.write_all(&msg).await.unwrap();
        self.writer.flush().await.unwrap();

        self.reader.read_exact(&mut msg).await.unwrap();
    }

    pub async fn init_download(&mut self) {
        loop {
            let msg = PeerMsg::read(&mut self.reader).await;
            if let PeerMsg::Bitfield(_) = msg {
                break;
            }
            dbg!(msg);
        }
        PeerMsg::Interested.write(&mut self.writer).await;

        loop {
            let msg = PeerMsg::read(&mut self.reader).await;
            if let PeerMsg::Unchoke = msg {
                break;
            }
            dbg!(msg);
        }
    }

    pub fn start_download_tasks(
        mut self,
        mut piece_req_receiver: UnboundedReceiver<PieceReq>,
        block_resp_sender: Sender<BlockResp>,
        block_size: u32,
    ) -> (JoinHandle<()>, JoinHandle<()>) {
        let buffer_size = 5;
        let (token_sender, mut token_receiver) = channel::<()>(buffer_size);

        let request_writer = spawn(async move {
            loop {
                let Some(piece) = piece_req_receiver.recv().await else {
                    return;
                };

                let blocks = piece.into_block_reqs(block_size);
                for block in blocks {
                    token_sender.send(()).await.unwrap();
                    let req = PeerMsg::from(block);
                    req.write(&mut self.writer).await;
                }
            }
        });

        let response_reader = spawn(async move {
            loop {
                if token_receiver.recv().await.is_none() {
                    return;
                };

                loop {
                    let msg = PeerMsg::read(&mut self.reader).await;
                    if let PeerMsg::Piece {
                        idx: _,
                        begin,
                        bytes,
                    } = msg
                    {
                        block_resp_sender
                            .send(BlockResp::new(begin, bytes))
                            .await
                            .unwrap();
                        break;
                    }
                }
            }
        });

        (request_writer, response_reader)
    }
}
