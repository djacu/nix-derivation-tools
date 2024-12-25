use nix_derivation_parser::derivations::parsers::parse_derivation;
use std::fs;
use winnow::Parser;

fn main() {
    let input =
        fs::read_to_string(
            "./src/vlv5v250k5daq2dnhj3bzn7p5dnsrg2f-nixos-system-massflash-24.05.20241009.d51c286.drv",
        ).expect("Invalid file");
    match parse_derivation::<()>.parse_next(&mut input.as_str()) {
        Ok(parsed) => {
            println!("Parsed: {parsed:#?}");
        },
        Err(err) => {
            eprintln!("Error parsing DerivationOutput: {err:?}");
        },
    };
}
