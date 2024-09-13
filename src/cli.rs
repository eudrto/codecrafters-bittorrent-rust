use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub s_command: SCommand,
}

#[derive(Subcommand)]
pub enum SCommand {
    Decode {
        bencoded_value: String,
    },
    Info {
        torrent_file_path: String,
    },
    Peers {
        torrent_file_path: String,
    },
    Handshake {
        torrent_file_path: String,
        peer_addr: String,
    },
}
