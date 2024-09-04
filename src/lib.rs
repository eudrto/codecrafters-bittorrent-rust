mod cli;

use clap::Parser;
use cli::{Cli, SCommand};
use serde_json;

fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    if encoded_value.chars().next().unwrap().is_digit(10) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<i64>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
        return serde_json::Value::String(string.to_string());
    } else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
}

pub fn run() {
    let cli = Cli::parse();

    match cli.s_command {
        SCommand::Decode { bencoded_value } => {
            let decoded_value = decode_bencoded_value(&bencoded_value);
            println!("{}", decoded_value.to_string());
        }
    }
}
