use crate::strings::types::StringFragment;

extern crate alloc;

use alloc::string::String;
use core::char::from_u32;
use core::num::ParseIntError;
use nom::{
    branch::alt,
    bytes::streaming::{
        is_not,
        take_while_m_n,
    },
    character::streaming::{
        char,
        multispace1,
    },
    combinator::{
        map,
        map_opt,
        map_res,
        value,
        verify,
    },
    error::{
        FromExternalError,
        ParseError,
    },
    multi::fold_many0,
    sequence::{
        delimited,
        preceded,
    },
    IResult,
};

// The string parsing functions are copied from github.com/rust-bakery/nom
// Originally from tag: 7.1.3 Specifically from this file:
// github.com/rust-bakery/nom/blob/7.1.3/examples/string.rs parser combinators are
// constructed from the bottom up: first we write parsers for the smallest
// elements (escaped characters), then combine them into larger parsers.
/// Parse a unicode sequence, of the form u{XXXX}, where XXXX is 1 to 6 hexadecimal
/// numerals. We will combine this later with `parse_escaped_char` to parse
/// sequences like \u{00AC}.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_unicode<'input, E>(input: &'input str) -> IResult<&'input str, char, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    // `take_while_m_n` parses between `m` and `n` bytes (inclusive) that match a
    // predicate. `parse_hex` here parses between 1 and 6 hexadecimal numerals.
    let parse_hex = take_while_m_n(1, 6, |character: char| character.is_ascii_hexdigit());

    // `preceded` takes a prefix parser, and if it succeeds, returns the result of the
    // body parser. In this case, it parses u{XXXX}.
    let parse_delimited_hex = preceded(
        char('u'),
        // `delimited` is like `preceded`, but it parses both a prefix and a suffix. It
        // returns the result of the middle parser. In this case, it parses {XXXX}, where
        // XXXX is 1 to 6 hex numerals, and returns XXXX
        delimited(char('{'), parse_hex, char('}')),
    );

    // `map_res` takes the result of a parser and applies a function that returns a
    // Result. In this case we take the hex bytes from parse_hex and attempt to
    // convert them to a u32.
    let parse_u32 = map_res(parse_delimited_hex, move |hex| u32::from_str_radix(hex, 16));

    // map_opt is like map_res, but it takes an Option instead of a Result. If the
    // function returns None, map_opt returns an error. In this case, because not all
    // u32 values are valid unicode code points, we have to fallibly convert to char
    // with from_u32.
    map_opt(parse_u32, from_u32)(input)
}

/// Parse an escaped character: \n, \t, \r, \u{00AC}, etc.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_escaped_char<'input, E>(input: &'input str) -> IResult<&'input str, char, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    preceded(
        char('\\'),
        // `alt` tries each parser in sequence, returning the result of the first
        // successful match
        alt((
            parse_unicode,
            // The `value` parser returns a fixed value (the first argument) if its parser
            // (the second argument) succeeds. In these cases, it looks for the marker
            // characters (n, r, t, etc) and returns the matching character (\n, \r, \t, etc).
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

/// Parse a backslash, followed by any amount of whitespace. This is used later to
/// discard any escaped whitespace.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_escaped_whitespace<
    'input,
    E: ParseError<&'input str>,
>(input: &'input str) -> IResult<&'input str, &'input str, E> {
    preceded(char('\\'), multispace1)(input)
}

/// Parse a non-empty block of text that doesn't include \ or "
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_literal<'input, E: ParseError<&'input str>>(input: &'input str) -> IResult<&'input str, &'input str, E> {
    // `is_not` parses a string of 0 or more characters that aren't one of the given
    // characters.
    let not_quote_slash = is_not("\"\\");

    // `verify` runs a parser, then runs a verification function on the output of the
    // parser. The verification function accepts out output only if it returns true.
    // In this case, we want to ensure that the output of is_not is non-empty.
    verify(not_quote_slash, |literal: &str| !literal.is_empty())(input)
}

/// Combine `parse_literal`, `parse_escaped_whitespace`, and `parse_escaped_char`
/// into a `StringFragment`.
#[expect(clippy::single_call_fn, reason = "Parser functions are not inlined for readability.")]
fn parse_fragment<'input, E>(input: &'input str) -> IResult<&'input str, StringFragment<'input>, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    alt((
        // The `map` combinator runs a parser, then applies a function to the output of
        // that parser.
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

/// Parse a string. Use a loop of `parse_fragment` and push all of the fragments
/// into an output string.
#[inline]
pub fn parse_string<'input, E>(input: &'input str) -> IResult<&'input str, String, E>
where
    E: ParseError<&'input str> + FromExternalError<&'input str, ParseIntError> {
    // fold_many0 is the equivalent of iterator::fold. It runs a parser in a loop, and
    // for each output value, calls a folding function on each output value.
    let build_string = fold_many0(
        // Our parser function– parses a single string fragment
        parse_fragment,
        // Our init value, an empty string
        String::new,
        // Our folding function. For each fragment, append the fragment to the string.
        |mut string, fragment| {
            match fragment {
                StringFragment::Literal(frag) => string.push_str(frag),
                StringFragment::EscapedChar(frag) => string.push(frag),
                StringFragment::EscapedWS => { },
            }
            string
        },
    );

    // Finally, parse the string. Note that, if `build_string` could accept a raw "
    // character, the closing delimiter " would never match. When using `delimited`
    // with a looping parser (like fold_many0), be sure that the loop won't
    // accidentally match your closing delimiter!
    delimited(char('"'), build_string, char('"'))(input)
}
