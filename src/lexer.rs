use crate::error::{Error, ParseErrorKind, Result};
use std::str::CharIndices;
use std::{fmt, iter::Peekable};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum LexItem {
    Indent(String),
    Ending(String),
    Comment(String),
    Directive {
        key: String,
        sep: String,
        value: String,
    },
}

impl LexItem {
    pub(crate) fn into_indent(self) -> Option<String> {
        match self {
            LexItem::Indent(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn into_ending(self) -> Option<String> {
        match self {
            LexItem::Ending(s) => Some(s),
            _ => None,
        }
    }

    pub(crate) fn into_comment(self) -> Option<String> {
        match self {
            LexItem::Comment(s) => Some(s),
            _ => None,
        }
    }
}

impl fmt::Display for LexItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexItem::Indent(s) => write!(f, "{s}"),
            LexItem::Ending(s) => write!(f, "{s}"),
            LexItem::Comment(s) => write!(f, "{s}"),
            LexItem::Directive { key, sep, value } => write!(f, "{key}{sep}{value}"),
        }
    }
}

pub(crate) struct Lexer<'a> {
    data: &'a str,
    iter: Peekable<CharIndices<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    pub(crate) fn new(data: &'a str) -> Self {
        Self {
            data,
            iter: data.char_indices().peekable(),
            line: 1,
            col: 1,
        }
    }

    #[inline]
    fn peek_next_char_offset(&mut self) -> usize {
        self.iter
            .peek()
            .map(|&(offset, _)| offset)
            .unwrap_or(self.data.len())
    }

    fn handle_indent(&mut self) -> LexItem {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        while let Some(&(offset, char)) = self.iter.peek() {
            // check new lines to avoid breaking the line counter
            if !char.is_whitespace() || char == '\n' || char == '\r' {
                break;
            }
            self.iter.next();
            self.col += 1;
            end = offset + char.len_utf8();
        }
        LexItem::Indent(self.data[start..end].to_string())
    }

    fn handle_comment(&mut self) -> LexItem {
        // except the first char to be '#'
        let start = self.peek_next_char_offset();
        let mut end = 0;
        while let Some(&(offset, char)) = self.iter.peek() {
            // consider a comment end when on a newline or when there is no more data
            if char == '\n' || char == '\r' {
                break;
            }
            self.iter.next();
            self.col += 1;
            end = offset + char.len_utf8();
        }

        LexItem::Comment(self.data[start..end].to_string())
    }

    fn handle_ending(&mut self) -> Result<LexItem> {
        match self.iter.next() {
            // handle single LF newline
            Some((offset, '\n')) => {
                self.line += 1;
                self.col = 1;
                Ok(LexItem::Ending(self.data[offset..offset + 1].to_string()))
            }

            // handle CRLF and improper format
            Some((offset, '\r')) => {
                self.col += 1;
                match self.iter.next() {
                    Some((_, '\n')) => {
                        self.line += 1;
                        self.col = 1;
                        Ok(LexItem::Ending(self.data[offset..offset + 2].to_string()))
                    }
                    Some((_, _)) | None => Err(Error::Parse {
                        line: self.line,
                        col: self.col,
                        kind: ParseErrorKind::UnterminatedCRLF,
                    }),
                }
            }

            // catchall for improper data format
            Some((_, _)) | None => Err(Error::Parse {
                line: self.line,
                col: self.col,
                kind: ParseErrorKind::InvalidParserUse,
            }),
        }
    }

    fn handle_field_key(&mut self) -> Result<&str> {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if key boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    return Err(Error::Parse {
                        line: self.line,
                        col: self.col,
                        kind: ParseErrorKind::KeyBoundaryNotFound,
                    });
                }
                // key boundary / separator territory
                Some((_, c)) if c.is_whitespace() || *c == '=' => {
                    break;
                }
                // key content
                Some((offset, c)) => {
                    end = offset + c.len_utf8();
                    self.col += 1;
                    self.iter.next();
                }
            }
        }
        Ok(&self.data[start..end])
    }

    fn handle_field_separator(&mut self) -> Result<&str> {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        let mut equal_seen = false;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if separator boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    return Err(Error::Parse {
                        line: self.line,
                        col: self.col,
                        kind: ParseErrorKind::SeparatorBoundaryNotFound,
                    });
                }

                // first time we see '='
                Some((offset, c @ '=')) if !equal_seen => {
                    equal_seen = true;
                    end = offset + c.len_utf8();
                    self.col += 1;
                    self.iter.next();
                }

                // error if we see '=' another time
                Some((_, '=')) => {
                    return Err(Error::Parse {
                        line: self.line,
                        col: self.col,
                        kind: ParseErrorKind::SeparatorHasMoreThanOneEqual,
                    });
                }

                // separator boundary
                Some((_, c)) if !c.is_whitespace() => {
                    break;
                }

                // separator content
                Some((offset, c)) => {
                    end = offset + c.len_utf8();
                    self.col += 1;
                    self.iter.next();
                }
            }
        }
        Ok(&self.data[start..end])
    }

    fn handle_field_value(&mut self) -> Result<&str> {
        let start_line = self.line;
        let start_col = self.col;
        let start = self.peek_next_char_offset();
        let mut end = 0;
        let mut has_consumed = false;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if value boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None if !has_consumed => {
                    return Err(Error::Parse {
                        line: self.line,
                        col: self.col,
                        kind: ParseErrorKind::ExpectedFieldValue,
                    });
                }

                // value boundary
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    break;
                }

                // key content
                Some((offset, c)) => {
                    has_consumed = true;
                    end = *offset + c.len_utf8();
                    self.col += 1;
                    self.iter.next();
                }
            }
        }

        let value_data = &self.data[start..end];

        // naively detect unclosed double quotes by checking if there is an odd count
        if value_data.chars().filter(|c| *c == '"').count() % 2 != 0 {
            return Err(Error::Parse {
                line: start_line,
                col: start_col,
                kind: ParseErrorKind::UnclosedDoubleQuote,
            });
        }
        Ok(value_data)
    }

    fn handle_directive(&mut self) -> Result<LexItem> {
        Ok(LexItem::Directive {
            key: self.handle_field_key()?.to_string(),
            sep: self.handle_field_separator()?.to_string(),
            value: self.handle_field_value()?.to_string(),
        })
    }

    /// <https://man7.org/linux/man-pages/man5/ssh_config.5.html>
    pub(crate) fn tokenize(mut self) -> Result<Vec<LexItem>> {
        let mut tokens = Vec::new();
        while let Some(&(_, c)) = self.iter.peek() {
            match c {
                // Comments
                '#' => tokens.push(self.handle_comment()),

                // Newlines
                '\n' | '\r' => tokens.push(self.handle_ending()?),

                // Indent
                c if c.is_whitespace() => tokens.push(self.handle_indent()),

                // Directive
                _ => {
                    tokens.push(self.handle_directive()?);
                }
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(data: &str) -> Vec<LexItem> {
        Lexer::new(data).tokenize().unwrap()
    }

    // --- single items ---

    #[test]
    fn parse_single_comment() {
        let data = "# this is a comment";
        assert_eq!(lex(data), vec![LexItem::Comment(data.into())]);
    }

    #[test]
    fn parse_indent() {
        let data = "      ";
        assert_eq!(lex(data), vec![LexItem::Indent(data.into())]);
    }

    #[test]
    fn parse_line_ending_lf() {
        assert_eq!(lex("\n"), vec![LexItem::Ending("\n".into())]);
    }

    #[test]
    fn parse_line_ending_crlf() {
        assert_eq!(lex("\r\n"), vec![LexItem::Ending("\r\n".into())]);
    }

    #[test]
    fn parse_line_ending_crlflf() {
        assert_eq!(
            lex("\r\n\n"),
            vec![LexItem::Ending("\r\n".into()), LexItem::Ending("\n".into()),]
        );
    }

    #[test]
    fn indent_stops_at_newline() {
        assert_eq!(
            lex("  \nHost x"),
            vec![
                LexItem::Indent("  ".into()),
                LexItem::Ending("\n".into()),
                LexItem::Directive {
                    key: "Host".into(),
                    sep: " ".into(),
                    value: "x".into(),
                },
            ]
        );
    }

    // --- directives ---

    #[test]
    fn directive_is_one_item() {
        assert_eq!(
            lex("Host my.host"),
            vec![LexItem::Directive {
                key: "Host".into(),
                sep: " ".into(),
                value: "my.host".into(),
            }]
        );
    }

    #[test]
    fn directive_accepts_every_separator_form() {
        for sep in [" ", "   ", "=", "  =", "=  ", "  =  ", " = "] {
            let data = format!("Host{sep}my.host");
            assert_eq!(
                lex(&data),
                vec![LexItem::Directive {
                    key: "Host".into(),
                    sep: sep.into(),
                    value: "my.host".into(),
                }],
                "separator {sep:?}"
            );
        }
    }

    #[test]
    fn directive_keeps_quoted_value() {
        assert_eq!(
            lex("Host=\"my.host\""),
            vec![LexItem::Directive {
                key: "Host".into(),
                sep: "=".into(),
                value: "\"my.host\"".into(),
            }]
        );
    }

    // --- errors ---

    #[test]
    fn err_on_line_ending_cr() {
        assert!(Lexer::new("\r").tokenize().is_err());
    }

    #[test]
    fn err_on_unclosed_double_quote() {
        assert!(Lexer::new("Host=\"test").tokenize().is_err());
    }

    #[test]
    fn err_on_key_two_equal_value() {
        assert!(Lexer::new("Host==my.host").tokenize().is_err());
    }

    #[test]
    fn error_points_at_the_offending_char() {
        // second '=' is the 6th char of line 2
        let err = Lexer::new("Host a\nPort==22\n").tokenize().unwrap_err();
        assert_eq!(
            err,
            Error::Parse {
                line: 2,
                col: 6,
                kind: ParseErrorKind::SeparatorHasMoreThanOneEqual,
            }
        );
    }

    // --- lossless ---

    #[test]
    fn display_round_trips_the_input() {
        let data = "# c\nHost = a\r\n\tPort=22\n\n   ";
        let round_tripped: String = lex(data).iter().map(LexItem::to_string).collect();
        assert_eq!(round_tripped, data);
    }
}
