use crate::derivations::types::{
    Derivation,
    DerivationInput,
    DerivationOutput,
};
use crate::strings::parsers::parse_string;

extern crate alloc;

use alloc::string::String;
use core::num::ParseIntError;
use std::collections::HashMap;
use std::path::PathBuf;
use winnow::{
    bytes::tag,
    combinator::opt,
    error::{
        FromExternalError,
        ParseError,
    },
    multi::{
        fold_many1,
        separated0,
        separated1,
    },
    sequence::{
        delimited,
        preceded,
        separated_pair,
    },
    IResult,
    Parser,
};

/// Parses a list of `DerivationOutput`s.
///
/// There must be at least one derivation output.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_derivation_outputs(input: &str) -> IResult<&str, HashMap<String, DerivationOutput>> {
    delimited(
        tag("["),
        fold_many1((parse_derivation_output, opt(tag(","))), HashMap::new, |mut map, ((key, value), _)| {
            map.insert(key, value);
            map
        }),
        tag("]"),
    )(input)
}

/// Parses a single `DerivationOutput`.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_derivation_output(input: &str) -> IResult<&str, (String, DerivationOutput)> {
    delimited(
        tag("("),
        Parser::map(
            (
                parse_string,
                preceded(tag(","), parse_string),
                preceded(tag(","), parse_string),
                preceded(tag(","), parse_string),
            ),
            |(key, path, hash_algo, hash)| {
                (key, DerivationOutput {
                    path: PathBuf::from(path),
                    hash_algo,
                    hash,
                })
            },
        ),
        tag(")"),
    )(input)
}

/// Parses a list of `DerivationInput`s.
///
/// There must be at least one derivation input.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_derivation_inputs(input: &str) -> IResult<&str, HashMap<PathBuf, DerivationInput>> {
    delimited(
        tag("["),
        fold_many1((parse_derivation_input, opt(tag(","))), HashMap::new, |mut map, ((key, value), _)| {
            map.insert(key, value);
            map
        }),
        tag("]"),
    )(input)
}

/// Parses a single `DerivationInput`.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_derivation_input(input: &str) -> IResult<&str, (PathBuf, DerivationInput)> {
    delimited(
        tag("("),
        separated_pair(
            parse_string,
            tag(","),
            delimited(tag("["), separated1(parse_string, tag(",")), tag("]")),
        ).map(|(key, value)| (PathBuf::from(key), DerivationInput { value })),
        tag(")"),
    )(
        input,
    )
}

/// Parses a list of source inputs.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_source_inputs<'input, E>(input: &'input str) -> IResult<&'input str, Vec<PathBuf>, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    delimited(tag("["), separated0(Parser::map(parse_string, PathBuf::from), tag(",")), tag("]"))(input)
}

/// Parses a system.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_system<'input, E>(input: &'input str) -> IResult<&'input str, String, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    parse_string(input)
}

/// Parses a builder.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_builder<'input, E>(input: &'input str) -> IResult<&'input str, PathBuf, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    parse_string.map(PathBuf::from).parse_next(input)
}

/// Parses a list of builder arguments.
///
/// This list can be empty.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_builder_args<'input, E>(input: &'input str) -> IResult<&'input str, Vec<String>, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    delimited(tag("["), separated0(parse_string, tag(",")), tag("]"))(input)
}

/// Parses a single environment variable.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_environment_variable<'input, E>(input: &'input str) -> IResult<&'input str, (String, String), E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    delimited(tag("("), separated_pair(parse_string, tag(","), parse_string), tag(")"))(input)
}

/// Parses a list of environment variables.
///
/// This list can be empty.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_environment_variables<'input, E>(input: &'input str) -> IResult<&'input str, Vec<(String, String)>, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    delimited(tag("["), separated0(parse_environment_variable, tag(",")), tag("]"))(input)
}

/// Parses a `Derivation`.
#[inline]
pub fn parse_derivation(input: &str) -> IResult<&str, Derivation> {
    delimited(
        tag("Derive("),
        (
            parse_derivation_outputs,
            preceded(tag(","), parse_derivation_inputs),
            preceded(tag(","), parse_source_inputs),
            preceded(tag(","), parse_system),
            preceded(tag(","), parse_builder),
            preceded(tag(","), parse_builder_args),
            preceded(tag(","), parse_environment_variables),
        ),
        tag(")"),
    )
        .map(|(outputs, input_drvs, input_srcs, system, builder, args, env)| Derivation {
            outputs,
            input_drvs,
            input_srcs,
            system,
            builder,
            args,
            env,
        })
        .parse_next(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use winnow::error::{
        ErrMode,
        ErrorKind,
    };

    #[test]
    fn release_packages() {
        let derivation_file_path =
            Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src/derivations/release_packages");
        let paths = fs::read_dir(derivation_file_path).unwrap();
        for path in paths {
            let drv_string = fs::read_to_string(path.expect("There should be files here!").path());
            assert!(parse_derivation(&drv_string.unwrap()).is_ok())
        }
    }

    #[test]
    fn release_packages_ca() {
        let derivation_file_path =
            Path::new(
                &std::env::var_os("CARGO_MANIFEST_DIR").unwrap(),
            ).join("src/derivations/release_packages_ca");
        let paths = fs::read_dir(derivation_file_path).unwrap();
        for path in paths {
            let drv_string = fs::read_to_string(path.expect("There should be files here!").path());
            assert!(parse_derivation(&drv_string.unwrap()).is_ok())
        }
    }

    #[test]
    fn misc_derivations() {
        let derivation_file_path =
            Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src/derivations/misc_derivations");
        let paths = fs::read_dir(derivation_file_path).unwrap();
        for path in paths {
            let drv_string = fs::read_to_string(path.expect("There should be files here!").path());
            assert!(parse_derivation(&drv_string.unwrap()).is_ok())
        }
    }

    #[test]
    fn derivation_output_all_empty() {
        assert_eq!(parse_derivation_output(r#"("","","","")"#), Ok(("", ("".to_string(), DerivationOutput {
            path: PathBuf::from(""),
            hash_algo: "".to_string(),
            hash: "".to_string(),
        }))));
    }

    #[test]
    fn derivation_output_minimal() {
        assert_eq!(
            parse_derivation_output(
                r#"("out","/nix/store/l5x91w2x83z33alsm5pmgl1gslbaqiyy-nixos-system-massflash-24.05.20241009.d51c286","","")"#,
            ),
            Ok(("", ("out".to_string(), DerivationOutput {
                path: PathBuf::from(
                    "/nix/store/l5x91w2x83z33alsm5pmgl1gslbaqiyy-nixos-system-massflash-24.05.20241009.d51c286",
                ),
                hash_algo: "".to_string(),
                hash: "".to_string(),
            })))
        );
    }

    #[test]
    fn derivation_outputs_empty() {
        assert_eq!(
            parse_derivation_outputs(r#"[]"#),
            Err(ErrMode::Backtrack(winnow::error::Error::from_error_kind("]", ErrorKind::Many1)))
        );
    }

    #[test]
    fn derivation_output_shadow() {
        assert_eq!(
            parse_derivation_outputs(
                concat!(
                    r#"["#,
                    r#"("dev","/nix/store/0fji8fg0z6gi3zyvsad7gxamx4ca2477-shadow-4.14.6-dev","","")"#,
                    r#","#,
                    r#"("man","/nix/store/9bzr2i2vvvjqfrbkrxm4j4zxq73im9nf-shadow-4.14.6-man","","")"#,
                    r#","#,
                    r#"("out","/nix/store/gwihsgkd13xmk8vwfn2k1nkdi9bys42x-shadow-4.14.6","","")"#,
                    r#","#,
                    r#"("su","/nix/store/w7lf813b5w0zrmh9sbrwm9xnnm1sh1d1-shadow-4.14.6-su","","")"#,
                    r#"]"#
                ),
            ),
            Ok(("", HashMap::from([("dev".to_string(), DerivationOutput {
                path: PathBuf::from("/nix/store/0fji8fg0z6gi3zyvsad7gxamx4ca2477-shadow-4.14.6-dev"),
                hash_algo: "".to_string(),
                hash: "".to_string(),
            }), ("man".to_string(), DerivationOutput {
                path: PathBuf::from("/nix/store/9bzr2i2vvvjqfrbkrxm4j4zxq73im9nf-shadow-4.14.6-man"),
                hash_algo: "".to_string(),
                hash: "".to_string(),
            }), ("out".to_string(), DerivationOutput {
                path: PathBuf::from("/nix/store/gwihsgkd13xmk8vwfn2k1nkdi9bys42x-shadow-4.14.6"),
                hash_algo: "".to_string(),
                hash: "".to_string(),
            }), ("su".to_string(), DerivationOutput {
                path: PathBuf::from("/nix/store/w7lf813b5w0zrmh9sbrwm9xnnm1sh1d1-shadow-4.14.6-su"),
                hash_algo: "".to_string(),
                hash: "".to_string(),
            })])))
        );
    }

    #[test]
    fn derivation_inputs_shadow() {
        assert_eq!(
            parse_derivation_inputs(
                concat!(
                    r#"["#,
                    r#"("/nix/store/2a4nqx30swmddxgd5f3y1h8gynwb1mp9-bison-3.8.2.drv",["out"])"#,
                    r#"("/nix/store/81l2lyg7hx6zwlb7yamncj1b2pbz5rj1-tcb-1.2.drv",["dev"])"#,
                    r#"("/nix/store/9jqhmw0ksi0gab01asfd8gfj3wv3ahg6-docbook-xsl-nons-1.79.2.drv",["out"])"#,
                    r#"("/nix/store/9vyx2lhbiq2c6jg6xz68whkl29qy60j2-autoreconf-hook.drv",["out"])"#,
                    r#"("/nix/store/b5gkdv8336qp2wx7qppmd54nl29y0zh4-libxcrypt-4.4.36.drv",["out"])"#,
                    r#"("/nix/store/d4rparlxpipwi3y717ijj917h0lbmrbj-glibc-2.39-52.drv",["bin"])"#,
                    r#"("/nix/store/dgj37ph9745jy0bnzfz2hl1x8yjhaawy-itstool-2.0.7.drv",["out"])"#,
                    r#"("/nix/store/hl008qyglrzrsyg59pc499jxaf1rvgjz-source.drv",["out"])"#,
                    r#"("/nix/store/hll9cxnh7mm2maiy06vbxl6zk2y65kvh-fix-implicit-getdef_bool.patch.drv",["out"])"#,
                    r#"("/nix/store/icld2xsizd7xabkfr396chagxcv7qaal-libxslt-1.1.39.drv",["dev"])"#,
                    r#"("/nix/store/kblxy5ggi81bli1vkz550vpvmy36wlbp-linux-pam-1.6.1.drv",["out"])"#,
                    r#"("/nix/store/lzc3r3m5yp5xj9qnbz56zrkq94d5hhsy-flex-2.6.4.drv",["out"])"#,
                    r#"("/nix/store/nz98jzc49vlkky3vpq5lwjxh94b207fh-pkg-config-wrapper-0.29.2.drv",["out"])"#,
                    r#"("/nix/store/pfkmysygw53mz830rhwfkadnzdxv96yw-libxml2-2.12.7.drv",["dev"])"#,
                    r#"("/nix/store/wkgn8l6fyq3avhcpw1caj2r1z9dsw4r0-docbook-xml-4.5.drv",["out"])"#,
                    r#"("/nix/store/wql9zbydwdr0nqxkm20crcbhn68wb4pc-stdenv-linux.drv",["out"])"#,
                    r#"("/nix/store/xzz7s4cc4bakhaavx3qyn10sl9w7x445-libbsd-0.11.8.drv",["dev"])"#,
                    r#"("/nix/store/ysv6wz83jkvg7d65j0js4bml9k0yc4sv-bash-5.2p32.drv",["out"])"#,
                    r#"]"#
                ),
            ),
            Ok(
                (
                    "",
                    HashMap::from(
                        [
                            (
                                PathBuf::from("/nix/store/2a4nqx30swmddxgd5f3y1h8gynwb1mp9-bison-3.8.2.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/81l2lyg7hx6zwlb7yamncj1b2pbz5rj1-tcb-1.2.drv"),
                                DerivationInput { value: vec!["dev".to_string()] },
                            ),
                            (
                                PathBuf::from(
                                    "/nix/store/9jqhmw0ksi0gab01asfd8gfj3wv3ahg6-docbook-xsl-nons-1.79.2.drv",
                                ),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/9vyx2lhbiq2c6jg6xz68whkl29qy60j2-autoreconf-hook.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/b5gkdv8336qp2wx7qppmd54nl29y0zh4-libxcrypt-4.4.36.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/d4rparlxpipwi3y717ijj917h0lbmrbj-glibc-2.39-52.drv"),
                                DerivationInput { value: vec!["bin".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/dgj37ph9745jy0bnzfz2hl1x8yjhaawy-itstool-2.0.7.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/hl008qyglrzrsyg59pc499jxaf1rvgjz-source.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from(
                                    "/nix/store/hll9cxnh7mm2maiy06vbxl6zk2y65kvh-fix-implicit-getdef_bool.patch.drv",
                                ),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/icld2xsizd7xabkfr396chagxcv7qaal-libxslt-1.1.39.drv"),
                                DerivationInput { value: vec!["dev".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/kblxy5ggi81bli1vkz550vpvmy36wlbp-linux-pam-1.6.1.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/lzc3r3m5yp5xj9qnbz56zrkq94d5hhsy-flex-2.6.4.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from(
                                    "/nix/store/nz98jzc49vlkky3vpq5lwjxh94b207fh-pkg-config-wrapper-0.29.2.drv",
                                ),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/pfkmysygw53mz830rhwfkadnzdxv96yw-libxml2-2.12.7.drv"),
                                DerivationInput { value: vec!["dev".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/wkgn8l6fyq3avhcpw1caj2r1z9dsw4r0-docbook-xml-4.5.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/wql9zbydwdr0nqxkm20crcbhn68wb4pc-stdenv-linux.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/xzz7s4cc4bakhaavx3qyn10sl9w7x445-libbsd-0.11.8.drv"),
                                DerivationInput { value: vec!["dev".to_string()] },
                            ),
                            (
                                PathBuf::from("/nix/store/ysv6wz83jkvg7d65j0js4bml9k0yc4sv-bash-5.2p32.drv"),
                                DerivationInput { value: vec!["out".to_string()] },
                            ),
                        ],
                    ),
                ),
            )
        )
    }
}
