use std::fmt;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum ParseErrorKind {
    KeyBoundaryNotFound,
    SeparatorBoundaryNotFound,
    SeparatorHasMoreThanOneEqual,
    ExpectedFieldValue,
    UnclosedDoubleQuote,
    UnterminatedCRLF,
    InvalidParserUse,
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let err_msg = match self {
            ParseErrorKind::KeyBoundaryNotFound => "key boundary not found",
            ParseErrorKind::SeparatorBoundaryNotFound => "separator boundary not found",
            ParseErrorKind::SeparatorHasMoreThanOneEqual => "separator has more than one '='",
            ParseErrorKind::ExpectedFieldValue => "field value not found",
            ParseErrorKind::UnclosedDoubleQuote => "unclosed double quote",
            ParseErrorKind::UnterminatedCRLF => "unterminated CRLF, expected LF",
            ParseErrorKind::InvalidParserUse => "unexpected input data",
        };
        write!(f, "{err_msg}")
    }
}

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Error {
    Parse {
        line: usize,
        col: usize,
        kind: ParseErrorKind,
    },
    InvalidIndent(String),
    InvalidSeparator(String),
    InvalidComment(String),
    InvalidKey(String),
    InvalidValue(String),
    NotASelector(String),
    UnexpectedSelector(String),
    NotCumulative(String),
    EmptyKey,
    EmptyValue,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse { line, col, kind } => write!(f, "{kind} at ln:{line} col:{col}"),
            Error::InvalidIndent(s) => write!(f, "invalid indent: {s:?}"),
            Error::InvalidSeparator(s) => write!(f, "invalid separator: {s:?}"),
            Error::InvalidComment(s) => write!(f, "invalid comment: {s:?}"),
            Error::InvalidKey(s) => write!(f, "invalid key: {s:?}"),
            Error::InvalidValue(s) => write!(f, "invalid value: {s:?}"),
            Error::NotASelector(s) => write!(f, "invalid key, not a selector: {s:?}"),
            Error::UnexpectedSelector(s) => write!(f, "unexpected selector: {s:?}"),
            Error::NotCumulative(s) => {
                write!(f, "FieldKey already exist and is not cumulative: {s:?}")
            }
            Error::EmptyKey => write!(f, "empty key"),
            Error::EmptyValue => write!(f, "empty value"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
