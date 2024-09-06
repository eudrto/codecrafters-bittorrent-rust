mod bencoding;
mod cli;

use bencoding::BValue;
use clap::Parser;
use cli::{Cli, SCommand};
use serde_json::json;

pub fn run() {
    let cli = Cli::parse();

    match cli.s_command {
        SCommand::Decode { bencoded_value } => {
            let bval = BValue::parse(bencoded_value.as_bytes());
            println!("{}", json!(bval));
        }
    }
}
