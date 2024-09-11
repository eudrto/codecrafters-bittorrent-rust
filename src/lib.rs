mod bencoding;
mod bytes_reader;
mod cli;
mod metainfo;

use std::fs;

use bencoding::to_json;
use clap::Parser;
use cli::{Cli, SCommand};
use metainfo::Metainfo;

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
    }
}
