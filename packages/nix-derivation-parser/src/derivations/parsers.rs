use crate::derivations::types::{Derivation, DerivationInput, DerivationOutput};
use crate::strings::parsers::parse_string;

use nom::{
    bytes::complete::tag,
    combinator::{all_consuming, map},
    multi::{separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
use std::path::PathBuf;
use std::string::String;

fn parse_derivation_outputs(input: &str) -> IResult<&str, Vec<DerivationOutput>> {
    delimited(
        tag("["),
        separated_list0(tag(","), parse_derivation_output),
        tag("]"),
    )(input)
}

// Parser for a single `DerivationOutput`
fn parse_derivation_output(input: &str) -> IResult<&str, DerivationOutput> {
    delimited(
        tag("("),
        map(
            tuple((
                parse_string,
                preceded(tag(","), parse_string),
                preceded(tag(","), parse_string),
                preceded(tag(","), parse_string),
            )),
            |(key, path, hash_algo, hash)| DerivationOutput {
                key,
                path: PathBuf::from(path),
                hash_algo,
                hash,
            },
        ),
        tag(")"),
    )(input)
}

fn parse_derivation_inputs(input: &str) -> IResult<&str, Vec<DerivationInput>> {
    delimited(
        tag("["),
        separated_list0(tag(","), parse_derivation_input),
        tag("]"),
    )(input)
}

fn parse_derivation_input(input: &str) -> IResult<&str, DerivationInput> {
    delimited(
        tag("("),
        map(
            separated_pair(
                parse_string,
                tag(","),
                delimited(tag("["), separated_list1(tag(","), parse_string), tag("]")),
            ),
            |(key, value)| DerivationInput {
                key: PathBuf::from(key),
                value,
            },
        ),
        tag(")"),
    )(input)
}

fn parse_source_inputs(input: &str) -> IResult<&str, Vec<PathBuf>> {
    delimited(
        tag("["),
        separated_list0(tag(","), map(parse_string, |x| PathBuf::from(x))),
        tag("]"),
    )(input)
}

fn parse_system(input: &str) -> IResult<&str, String> {
    parse_string(input)
}

fn parse_builder(input: &str) -> IResult<&str, PathBuf> {
    map(parse_string, |x| PathBuf::from(x))(input)
}

fn parse_builder_args(input: &str) -> IResult<&str, Vec<String>> {
    delimited(tag("["), separated_list0(tag(","), parse_string), tag("]"))(input)
}

fn parse_environment_variable(input: &str) -> IResult<&str, (String, String)> {
    delimited(
        tag("("),
        separated_pair(parse_string, tag(","), parse_string),
        tag(")"),
    )(input)
}

fn parse_environment_variables(input: &str) -> IResult<&str, Vec<(String, String)>> {
    delimited(
        tag("["),
        separated_list0(tag(","), parse_environment_variable),
        tag("]"),
    )(input)
}

pub fn parse_derivation(input: &str) -> IResult<&str, Derivation> {
    map(
        all_consuming(delimited(
            tag("Derive("),
            tuple((
                parse_derivation_outputs,
                preceded(tag(","), parse_derivation_inputs),
                preceded(tag(","), parse_source_inputs),
                preceded(tag(","), parse_system),
                preceded(tag(","), parse_builder),
                preceded(tag(","), parse_builder_args),
                preceded(tag(","), parse_environment_variables),
            )),
            tag(")"),
        )),
        |(outputs, input_drvs, input_srcs, system, builder, args, env)| Derivation {
            outputs,
            input_drvs,
            input_srcs,
            system,
            builder,
            args,
            env,
        },
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivation_output_all_empty() {
        assert_eq!(
            parse_derivation_output(r#"("","","","")"#),
            Ok((
                "",
                DerivationOutput {
                    key: "".to_string(),
                    path: PathBuf::from(""),
                    hash_algo: "".to_string(),
                    hash: "".to_string()
                }
            ))
        );
    }

    #[test]
    fn derivation_output_minimal() {
        assert_eq!(
            parse_derivation_output(
                r#"("out","/nix/store/l5x91w2x83z33alsm5pmgl1gslbaqiyy-nixos-system-massflash-24.05.20241009.d51c286","","")"#
            ),
            Ok((
                "",
                DerivationOutput {
                    key: "out".to_string(),
                    path: PathBuf::from("/nix/store/l5x91w2x83z33alsm5pmgl1gslbaqiyy-nixos-system-massflash-24.05.20241009.d51c286"),
                    hash_algo: "".to_string(),
                    hash: "".to_string()
                }
            ))
        );
    }
}
