mod bdict;
mod bvalue;
mod binteger;
mod blist;
mod bstring;
mod bytes_reader;
mod cli;

use bvalue::BValue;
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
