use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::streaming::{is_not, take_while_m_n},
    character::streaming::{char, multispace1},
    combinator::{all_consuming, map, map_opt, map_res, value, verify},
    error::{FromExternalError, ParseError},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, tuple},
    IResult,
};
use std::fs;
use std::path::PathBuf;
use std::string::String;

// The string parsing functions are copied from github.com/rust-bakery/nom
// Originally from tag: 7.1.3
// Specifically from this file: github.com/rust-bakery/nom/blob/7.1.3/examples/string.rs

// parser combinators are constructed from the bottom up:
// first we write parsers for the smallest elements (escaped characters),
// then combine them into larger parsers.

/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6
/// hexadecimal numerals. We will combine this later with parse_escaped_char
/// to parse sequences like \u{00AC}.
fn parse_unicode<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    // `take_while_m_n` parses between `m` and `n` bytes (inclusive) that match
    // a predicate. `parse_hex` here parses between 1 and 6 hexadecimal numerals.
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());

    // `preceded` takes a prefix parser, and if it succeeds, returns the result
    // of the body parser. In this case, it parses u{XXXX}.
    let parse_delimited_hex = preceded(
        char('u'),
        // `delimited` is like `preceded`, but it parses both a prefix and a suffix.
        // It returns the result of the middle parser. In this case, it parses
        // {XXXX}, where XXXX is 1 to 6 hex numerals, and returns XXXX
        delimited(char('{'), parse_hex, char('}')),
    );

    // `map_res` takes the result of a parser and applies a function that returns
    // a Result. In this case we take the hex bytes from parse_hex and attempt to
    // convert them to a u32.
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

    // map_opt is like map_res, but it takes an Option instead of a Result. If
    // the function returns None, map_opt returns an error. In this case, because
    // not all u32 values are valid unicode code points, we have to fallibly
    // convert to char with from_u32.
    map_opt(parse_u32, |value| std::char::from_u32(value))(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
fn parse_escaped_char<'a, E>(input: &'a str) -> IResult<&'a str, char, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    preceded(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of
        // the first successful match
        alt((
            parse_unicode,
            // The `value` parser returns a fixed value (the first argument) if its
            // parser (the second argument) succeeds. In these cases, it looks for
            // the marker characters (n, r, t, etc) and returns the matching
            // character (\n, \r, \t, etc).
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

/// Parse a backslash, followed by any amount of whitespace. This is used later
/// to discard any escaped whitespace.
fn parse_escaped_whitespace<'a, E: ParseError<&'a str>>(
    input: &'a str,
) -> IResult<&'a str, &'a str, E> {
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
fn parse_literal<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, &'a str, E> {
    // `is_not` parses a string of 0 or more characters that aren't one of the
    // given characters.
    let not_quote_slash = is_not("\"\\");

    // `verify` runs a parser, then runs a verification function on the output of
    // the parser. The verification function accepts out output only if it
    // returns true. In this case, we want to ensure that the output of is_not
    // is non-empty.
    verify(not_quote_slash, |s: &str| !s.is_empty())(input)
}

/// A string fragment contains a fragment of a string being parsed: either
/// a non-empty Literal (a series of non-escaped characters), a single
/// parsed escaped character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StringFragment<'a> {
    Literal(&'a str),
    EscapedChar(char),
    EscapedWS,
}

/// Combine parse_literal, parse_escaped_whitespace, and parse_escaped_char
/// into a StringFragment.
fn parse_fragment<'a, E>(input: &'a str) -> IResult<&'a str, StringFragment<'a>, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    alt((
        // The `map` combinator runs a parser, then applies a function to the output
        // of that parser.
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// Parse a string. Use a loop of parse_fragment and push all of the fragments
/// into an output string.
fn parse_string<'a, E>(input: &'a str) -> IResult<&'a str, String, E>
where
    E: ParseError<&'a str> + FromExternalError<&'a str, std::num::ParseIntError>,
{
    // fold_many0 is the equivalent of iterator::fold. It runs a parser in a loop,
    // and for each output value, calls a folding function on each output value.
    let build_string = fold_many0(
        // Our parser function– parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the
        // string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(s) => string.push_str(s),
                StringFragment::EscapedChar(c) => string.push(c),
                StringFragment::EscapedWS => {}
            }
            string
        },
    );

    // Finally, parse the string. Note that, if `build_string` could accept a raw
    // " character, the closing delimiter " would never match. When using
    // `delimited` with a looping parser (like fold_many0), be sure that the
    // loop won't accidentally match your closing delimiter!
    delimited(char('"'), build_string, char('"'))(input)
}

fn parse_derivation_outputs(input: &str) -> IResult<&str, Vec<DerivationOutput>> {
    delimited(tag("["), many0(parse_derivation_output), tag("]"))(input)
}

#[derive(Debug, PartialEq)]
struct DerivationOutput {
    key: String,
    path: PathBuf,
    hash_algo: String,
    hash: String,
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

#[derive(Debug, PartialEq)]
struct DerivationInput {
    key: PathBuf,
    value: Vec<String>,
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

#[derive(Debug, PartialEq)]
struct Derivation {
    outputs: Vec<DerivationOutput>,
    input_drvs: Vec<DerivationInput>,
    input_srcs: Vec<PathBuf>,
    system: String,
    builder: PathBuf,
    args: Vec<String>,
    env: Vec<(String, String)>,
}

fn parse_derivation(input: &str) -> IResult<&str, Derivation> {
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

fn main() {
    let input = fs::read_to_string(
        "./src/vlv5v250k5daq2dnhj3bzn7p5dnsrg2f-nixos-system-massflash-24.05.20241009.d51c286.drv",
    )
    .unwrap();

    match parse_derivation(&input) {
        Ok((remaining, parsed)) => {
            println!("Parsed: {:#?}", parsed);
            println!("Remaining: {:#?}", remaining);
        }
        Err(err) => {
            eprintln!("Error parsing DerivationOutput: {:?}", err);
        }
    };
}
