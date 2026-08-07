use crate::error::{Error, ParseErrorKind, Result};
use std::str::CharIndices;
use std::{fmt, iter::Peekable};

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TokenKind {
    WhiteSpace,
    LineEnding,
    Comment,
    FieldKey,
    FieldSeparator,
    FieldValue,
}

#[derive(Debug, PartialEq, Copy, Clone)]
struct Position {
    line: usize,
    col: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    kind: TokenKind,
    pub(crate) data: String,
    pos: Option<Position>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

impl Token {
    pub(crate) fn synthetic(kind: TokenKind, data: String) -> Self {
        Token {
            kind,
            data,
            pos: None,
        }
    }
}

struct Lexer<'a> {
    data: &'a str,
    iter: Peekable<CharIndices<'a>>,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(data: &'a str) -> Self {
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

    /// 1 based (line, column) of the next char to be consumed
    fn position(&self) -> Position {
        Position {
            line: self.line,
            col: self.col,
        }
    }

    fn handle_whitespace(&mut self) -> Token {
        let start_pos = self.position();

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
        Token {
            kind: TokenKind::WhiteSpace,
            data: self.data[start..end].to_string(),
            pos: Some(start_pos),
        }
    }

    fn handle_comment(&mut self) -> Token {
        let start_pos = self.position();

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

        Token {
            kind: TokenKind::Comment,
            data: self.data[start..end].to_string(),
            pos: Some(start_pos),
        }
    }

    fn handle_newline(&mut self) -> Result<Token> {
        let start_pos = self.position();

        match self.iter.next() {
            // handle single LF newline
            Some((offset, '\n')) => {
                self.line += 1;
                self.col = 1;
                Ok(Token {
                    kind: TokenKind::LineEnding,
                    data: self.data[offset..offset + 1].to_string(),
                    pos: Some(start_pos),
                })
            }

            // handle CRLF and improper format
            Some((offset, '\r')) => {
                self.col += 1;
                match self.iter.next() {
                    Some((_, '\n')) => {
                        self.line += 1;
                        self.col = 1;
                        Ok(Token {
                            kind: TokenKind::LineEnding,
                            data: self.data[offset..offset + 2].to_string(),
                            pos: Some(start_pos),
                        })
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

    fn handle_field_key(&mut self) -> Result<Token> {
        let start_pos = self.position();

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
        Ok(Token {
            kind: TokenKind::FieldKey,
            data: self.data[start..end].to_string(),
            pos: Some(start_pos),
        })
    }

    fn handle_field_separator(&mut self) -> Result<Token> {
        let start_pos = self.position();

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
        Ok(Token {
            kind: TokenKind::FieldSeparator,
            data: self.data[start..end].to_string(),
            pos: Some(start_pos),
        })
    }

    fn handle_field_value(&mut self) -> Result<Token> {
        let start_pos = self.position();

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

        let value_data = self.data[start..end].to_string();

        // naively detect unclosed double quotes by checking if there is an odd count
        if value_data.chars().filter(|c| *c == '"').count() % 2 != 0 {
            return Err(Error::Parse {
                line: start_pos.line,
                col: start_pos.col,
                kind: ParseErrorKind::UnclosedDoubleQuote,
            });
        }
        Ok(Token {
            kind: TokenKind::FieldValue,
            data: value_data,
            pos: Some(start_pos),
        })
    }

    /// <https://man7.org/linux/man-pages/man5/ssh_config.5.html>
    fn tokenize(mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some(&(_, c)) = self.iter.peek() {
            match c {
                // Comments
                '#' => tokens.push(self.handle_comment()),

                // Newlines
                '\n' | '\r' => tokens.push(self.handle_newline()?),

                // Whitespaces
                c if c.is_whitespace() => tokens.push(self.handle_whitespace()),

                // Other: key + separator + value
                _ => {
                    tokens.push(self.handle_field_key()?);
                    tokens.push(self.handle_field_separator()?);
                    tokens.push(self.handle_field_value()?);
                }
            }
        }
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_comment() {
        let data = "# this is a comment";
        let lexer = Lexer::new(data);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_whitespace() {
        let data = "      ";
        let lexer = Lexer::new(data);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::WhiteSpace);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_lf() {
        let data = "\n";
        let lexer = Lexer::new(data);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_crlf() {
        let data = "\r\n";
        let lexer = Lexer::new(data);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_crlflf() {
        let data = "\r\n\n";
        let lexer = Lexer::new(data);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, "\r\n");
        assert_eq!(tokens[1].kind, TokenKind::LineEnding);
        assert_eq!(tokens[1].data, "\n");
    }

    #[test]
    fn err_on_line_ending_cr() {
        let data = "\r";
        let lexer = Lexer::new(data);
        let err = lexer.tokenize();
        assert!(err.is_err());
    }

    #[test]
    fn parse_key() {
        let key = "Host";
        let sep = " ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
    }

    #[test]
    fn parse_key_single_whitespace_value() {
        let key = "Host";
        let sep = " ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_multiple_whitespace_value() {
        let key = "Host";
        let sep = "   ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_equal_value() {
        let key = "Host";
        let sep = "=";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_sep_quoted_value() {
        let key = "Host";
        let sep = "=";
        let value = "\"my.host\"";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn err_on_unclosed_double_quote() {
        let key = "Host";
        let sep = "=";
        let value = "\"test";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let err = lexer.tokenize();
        assert!(err.is_err());
    }

    #[test]
    fn parse_key_equal_whitespace_value() {
        let key = "Host";
        let sep = "=  ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_whitespace_equal_value() {
        let key = "Host";
        let sep = "  =";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_equal_whitespace_equal_value() {
        let key = "Host";
        let sep = "  =  ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::FieldKey);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::FieldSeparator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::FieldValue);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn err_on_key_two_equal_value() {
        let key = "Host";
        let sep = "==";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let lexer = Lexer::new(&data);
        let err = lexer.tokenize();
        assert!(err.is_err());
    }

    #[test]
    fn whitespace_stops_at_newline() {
        let tokens = Lexer::new("  \nHost x").tokenize().unwrap();
        assert_eq!(tokens[0].data, "  ");
        assert_eq!(tokens[1].kind, TokenKind::LineEnding);
    }

    #[test]
    fn token_positions_point_at_token_start() {
        let tokens = Lexer::new("Host a\nUser b").tokenize().unwrap();

        let positions: Vec<(usize, usize)> = tokens
            .iter()
            .map(|t| {
                let p = t.pos.expect("lexed tokens carry a position");
                (p.line, p.col)
            })
            .collect();

        assert_eq!(
            positions,
            vec![
                (1, 1), // FieldKey       "Host"  <- start of line 1
                (1, 5), // FieldSeparator " "
                (1, 6), // FieldValue     "a"
                (1, 7), // LineEnding     "\n"
                (2, 1), // FieldKey       "User"  <- start of line 2
                (2, 5), // FieldSeparator " "
                (2, 6), // FieldValue     "b"
            ]
        );
    }

    #[test]
    fn crlf_does_not_shift_the_next_line() {
        // the CR branch increments col before consuming '\n', so the reset is worth pinning
        let tokens = Lexer::new("Host a\r\nUser b").tokenize().unwrap();
        let key = &tokens[4];
        assert_eq!(key.kind, TokenKind::FieldKey);
        assert_eq!(key.data, "User");
        assert_eq!(key.pos, Some(Position { line: 2, col: 1 }));
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
}
