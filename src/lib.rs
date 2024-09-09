mod bencoding;
mod bytes_reader;
mod cli;
mod metainfo;

use std::fs;

use bencoding::BValue;
use clap::Parser;
use cli::{Cli, SCommand};
use metainfo::Metainfo;
use serde_json::json;

pub fn run() {
    let cli = Cli::parse();

    match cli.s_command {
        SCommand::Decode { bencoded_value } => {
            let bval = BValue::decode(bencoded_value.as_bytes());
            println!("{}", json!(bval));
        }
        SCommand::Info { torrent_file_path } => {
            let metainfo_bytes = fs::read(torrent_file_path).unwrap();
            let metainfo = Metainfo::new(&metainfo_bytes);

            println!("Tracker URL: {}", metainfo.get_tracker());
            println!("Length: {}", metainfo.get_length());
            println!("Info Hash: {}", metainfo.get_info_hash())
        }
    }
}
