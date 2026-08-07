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
    InvalidLineEnding(String),
    EmptyKey,
    EmptyValue,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse { line, col, kind } => write!(f, "{kind} at ln:{line} col:{col}"),
            Error::InvalidIndent(s) => write!(f, "invalid indent: {s:?}"),
            Error::InvalidLineEnding(s) => write!(f, "invalid line ending: {s:?}"),
            Error::InvalidSeparator(s) => write!(f, "invalid separator: {s:?}"),
            Error::EmptyKey => write!(f, "empty key"),
            Error::EmptyValue => write!(f, "empty value"),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
