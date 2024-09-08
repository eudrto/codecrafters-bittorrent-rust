mod bstring;
mod bencoding;
mod bytes_reader;
mod cli;

use bencoding::BValue;
use clap::Parser;
use cli::{Cli, SCommand};
use serde_json::json;

pub fn run() {
    let cli = Cli::parse();

    match cli.s_command {
        SCommand::Decode { bencoded_value } => {
            let bval = BValue::decode(bencoded_value.as_bytes());
            println!("{}", json!(bval));
        }
    }
}
