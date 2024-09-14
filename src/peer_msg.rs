use std::{
    io::{BufReader, BufWriter, Read, Write},
    net::TcpStream,
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
    Piece {
        idx: u32,
        begin: u32,
    },
    #[allow(dead_code)]
    Unknown(Vec<u8>),
}

impl PeerMsg {
    pub fn read(reader: &mut BufReader<&TcpStream>) -> Self {
        let length = read_u32(reader);
        if length == 0 {
            panic!("peer message length is 0");
        }

        let id = read_byte(reader);
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
                reader.read_exact(&mut buf).unwrap();
                Self::Bitfield(buf)
            }
            6 => {
                if length != 13 {
                    panic!("request length: {}", length);
                }
                Self::Request {
                    idx: read_u32(reader),
                    begin: read_u32(reader),
                    length: read_u32(reader),
                }
            }
            7 => {
                if length < 9 {
                    panic!("piece length: {}", length);
                }
                Self::Piece {
                    idx: read_u32(reader),
                    begin: read_u32(reader),
                }
            }
            _ => {
                let mut buf = vec![0; length as usize - 1];
                reader.read_exact(&mut buf).unwrap();
                Self::Unknown(buf)
            }
        }
    }

    pub fn write(&self, writer: &mut BufWriter<&TcpStream>) {
        match self {
            Self::Unchoke => unimplemented!(),
            Self::Interested => {
                let id = 2;
                write(writer, id, &[0; 0]);
            }
            Self::Bitfield(_) => unimplemented!(),
            Self::Request { idx, begin, length } => {
                let id = 6;
                write(writer, id, &[*idx, *begin, *length]);
            }
            Self::Piece { .. } => unimplemented!(),
            Self::Unknown { .. } => unimplemented!(),
        }
    }
}

fn read_byte(reader: &mut BufReader<&TcpStream>) -> u8 {
    let mut buf = [0; 1];
    reader.read_exact(&mut buf).unwrap();
    u8::from_be_bytes(buf.try_into().unwrap())
}

fn read_u32(reader: &mut BufReader<&TcpStream>) -> u32 {
    let mut buf = [0; 4];
    reader.read_exact(&mut buf).unwrap();
    u32::from_be_bytes(buf.try_into().unwrap())
}

fn write(writer: &mut BufWriter<&TcpStream>, id: u8, rest: &[u32]) {
    let mut msg = [0; 17];
    let length = 1 + 4 * rest.len();
    msg[..4].copy_from_slice(&(length as u32).to_be_bytes());
    msg[4] = id;
    for (idx, val) in rest.iter().enumerate() {
        let start = 5 + idx * 4;
        let end = start + 4;
        msg[start..end].copy_from_slice(&val.to_be_bytes());
    }
    writer.write_all(&msg[..4 + length]).unwrap();
    writer.flush().unwrap();
}
