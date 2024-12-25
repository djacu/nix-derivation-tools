/// A string fragment contains a fragment of a string being parsed: either a
/// non-empty Literal (a series of non-escaped characters), a single parsed escaped
/// character, or a block of escaped whitespace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StringFragment<'input> {
    Literal(&'input str),
    EscapedChar(char),
    EscapedWS,
}
