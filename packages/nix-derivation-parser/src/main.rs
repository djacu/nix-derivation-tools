use nix_derivation_parser::derivations::parsers::parse_derivation;
use std::fs;

fn main() {
    let input =
        fs::read_to_string(
            "./src/vlv5v250k5daq2dnhj3bzn7p5dnsrg2f-nixos-system-massflash-24.05.20241009.d51c286.drv",
        ).unwrap();
    match parse_derivation(&input) {
        Ok((remaining, parsed)) => {
            println!("Parsed: {:#?}", parsed);
            println!("Remaining: {:#?}", remaining);
        },
        Err(err) => {
            eprintln!("Error parsing DerivationOutput: {:?}", err);
        },
    };
}
