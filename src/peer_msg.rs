use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

#[derive(Debug)]
pub enum PeerMsg {
    Unchoke,
    Interested,
    #[allow(dead_code)]
    Bitfield(Vec<u8>),
    Request {
        idx: u32,
        begin: u32,
        length: u32,
    },
    #[allow(dead_code)]
    Piece {
        idx: u32,
        begin: u32,
        bytes: Vec<u8>,
    },
    #[allow(dead_code)]
    Unknown(Vec<u8>),
}

impl PeerMsg {
    pub async fn read(reader: &mut BufReader<OwnedReadHalf>) -> Self {
        let length = read_u32(reader).await;
        if length == 0 {
            panic!("peer message length is 0");
        }

        let id = read_byte(reader).await;
        match id {
            1 => {
                if length != 1 {
                    panic!("unchoke length: {}", length);
                }
                Self::Unchoke
            }
            2 => {
                if length != 1 {
                    panic!("interested length: {}", length);
                }
                Self::Interested
            }
            5 => {
                let mut buf = vec![0; length as usize - 1];
                reader.read_exact(&mut buf).await.unwrap();
                Self::Bitfield(buf)
            }
            6 => {
                if length != 13 {
                    panic!("request length: {}", length);
                }
                Self::Request {
                    idx: read_u32(reader).await,
                    begin: read_u32(reader).await,
                    length: read_u32(reader).await,
                }
            }
            7 => {
                if length < 9 {
                    panic!("piece length: {}", length);
                }
                let idx = read_u32(reader).await;
                let begin = read_u32(reader).await;
                let mut bytes = vec![0; length as usize - 9];
                reader.read_exact(&mut bytes).await.unwrap();
                Self::Piece { idx, begin, bytes }
            }
            _ => {
                let mut buf = vec![0; length as usize - 1];
                reader.read_exact(&mut buf).await.unwrap();
                Self::Unknown(buf)
            }
        }
    }

    pub async fn write(&self, writer: &mut BufWriter<OwnedWriteHalf>) {
        match self {
            Self::Unchoke => unimplemented!(),
            Self::Interested => {
                let id = 2;
                write(writer, id, &[0; 0]).await;
            }
            Self::Bitfield(_) => unimplemented!(),
            Self::Request { idx, begin, length } => {
                let id = 6;
                write(writer, id, &[*idx, *begin, *length]).await;
            }
            Self::Piece { .. } => unimplemented!(),
            Self::Unknown { .. } => unimplemented!(),
        }
    }
}

async fn read_byte(reader: &mut BufReader<OwnedReadHalf>) -> u8 {
    let mut buf = [0; 1];
    reader.read_exact(&mut buf).await.unwrap();
    u8::from_be_bytes(buf.try_into().unwrap())
}

async fn read_u32(reader: &mut BufReader<OwnedReadHalf>) -> u32 {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf).await.unwrap();
    u32::from_be_bytes(buf.try_into().unwrap())
}

async fn write(writer: &mut BufWriter<OwnedWriteHalf>, id: u8, rest: &[u32]) {
    let mut msg = [0; 17];
    let length = 1 + 4 * rest.len();
    msg[..4].copy_from_slice(&(length as u32).to_be_bytes());
    msg[4] = id;
    for (idx, val) in rest.iter().enumerate() {
        let start = 5 + idx * 4;
        let end = start + 4;
        msg[start..end].copy_from_slice(&val.to_be_bytes());
    }
    writer.write_all(&msg[..4 + length]).await.unwrap();
    writer.flush().await.unwrap();
}
