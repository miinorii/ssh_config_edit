use std::{iter::Peekable, fmt};
use std::str::CharIndices;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind {
    WhiteSpace,
    LineEnding,
    Comment,
    FieldKey,
    FieldSeparator,
    FieldValue,
}

#[derive(Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub data: String,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.data)
    }
}

pub struct Lexer<'a> {
    data: &'a str,
    iter: Peekable<CharIndices<'a>>,
    line: usize,
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a str) -> Self {
        Self {
            data: data,
            iter: data.char_indices().peekable(),
            line: 1,
            pos: 0,
        }
    }

    #[inline]
    fn peek_next_char_offset(&mut self) -> usize {
        return self.iter.peek().map(|&(o, _)| o).unwrap_or(self.data.len());
    }

    fn handle_whitespace(&mut self) -> Token {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        while let Some(&(offset, char)) = self.iter.peek() {
            // check new lines to avoid breaking the line counter
            if !char.is_whitespace() || char == '\n' || char == '\r' {
                break;
            }
            self.iter.next();
            self.pos += 1;
            end = offset + char.len_utf8();
        }
        return Token {
            kind: TokenKind::WhiteSpace,
            data: self.data[start..end].to_string(),
        };
    }

    fn handle_comment(&mut self) -> Token {
        // except the first char to be '#'
        let start = self.peek_next_char_offset();
        let mut end = 0;
        while let Some(&(offset, char)) = self.iter.peek() {
            // consider a comment end when on a newline or when there is no more data
            if char == '\n' || char == '\r' {
                break;
            }
            self.iter.next();
            self.pos += 1;
            end = offset + char.len_utf8();
        }

        return Token {
            kind: TokenKind::Comment,
            data: self.data[start..end].to_string(),
        };
    }

    fn handle_newline(&mut self) -> Result<Token, String> {
        match self.iter.next() {
            // handle single LF newline
            Some((offset, '\n')) => {
                self.line += 1;
                self.pos = 1;
                return Ok(Token {
                    kind: TokenKind::LineEnding,
                    data: self.data[offset..offset + 1].to_string(),
                });
            }

            // handle CRLF and improper format
            Some((offset, '\r')) => {
                self.pos += 1;
                match self.iter.next() {
                    Some((_, '\n')) => {
                        self.line += 1;
                        self.pos = 1;
                        return Ok(Token {
                            kind: TokenKind::LineEnding,
                            data: self.data[offset..offset + 2].to_string(),
                        });
                    }
                    Some((_, _)) | None => {
                        return Err(format!(
                            "at ln:{} pos:{}, expected '\n'",
                            self.line,
                            self.pos + 1,
                        ));
                    }
                }
            }

            // catchall for improper data format
            Some((_, _)) | None => {
                return Err(format!(
                    "at ln:{} pos:{}, improper data format",
                    self.line, self.pos
                ));
            }
        }
    }

    fn handle_field_key(&mut self) -> Result<Token, String> {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if key boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    return Err(format!(
                        "at ln:{} pos:{}, key boundary not found",
                        self.line,
                        self.pos + 1
                    ));
                }
                // key boundary / separator territory
                Some((_, c)) if c.is_whitespace() || *c == '=' => {
                    break;
                }
                // key content
                Some((offset, c)) => {
                    end = offset + c.len_utf8();
                    self.pos += 1;
                    self.iter.next();
                }
            }
        }
        return Ok(Token {
            kind: TokenKind::FieldKey,
            data: self.data[start..end].to_string(),
        });
    }

    fn handle_field_separator(&mut self) -> Result<Token, String> {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        let mut equal_seen = false;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if separator boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    return Err(format!(
                        "at ln:{} pos:{}, separator boundary not found",
                        self.line,
                        self.pos + 1
                    ));
                }

                // first time we see '='
                Some((offset, c @ '=')) if !equal_seen => {
                    equal_seen = true;
                    end = offset + c.len_utf8();
                    self.pos += 1;
                    self.iter.next();
                }

                // error if we see '=' another time
                Some((_, '=')) => {
                    return Err(format!(
                        "at ln:{} pos:{}, separator contain two '='",
                        self.line,
                        self.pos + 1
                    ));
                }

                // separator boundary
                Some((_, c)) if !c.is_whitespace() => {
                    break;
                }

                // separator content
                Some((offset, c)) => {
                    end = offset + c.len_utf8();
                    self.pos += 1;
                    self.iter.next();
                }
            }
        }
        return Ok(Token {
            kind: TokenKind::FieldSeparator,
            data: self.data[start..end].to_string(),
        });
    }

    fn handle_field_value(&mut self) -> Result<Token, String> {
        let start = self.peek_next_char_offset();
        let mut end = 0;
        let mut has_consumed = false;
        loop {
            let next_data = self.iter.peek();
            match next_data {
                // end if value boundary can't be detected (improper file format)
                Some((_, '\r')) | Some((_, '\n')) | None if !has_consumed => {
                    return Err(format!(
                        "at ln:{} pos:{}, value not provided",
                        self.line,
                        self.pos + 1
                    ));
                }

                // value boundary
                Some((_, '\r')) | Some((_, '\n')) | None => {
                    break;
                }

                // key content
                Some((offset, c)) => {
                    has_consumed = true;
                    end = *offset + c.len_utf8();
                    self.pos += 1;
                    self.iter.next();
                }
            }
        }

        let value_data = self.data[start..end].to_string();

        // naively detect unclosed double quotes by checking if there is an odd count
        if value_data.chars().filter(|c| *c == '"').count() % 2 != 0 {
            return Err(format!(
                "at ln:{} pos:{}, unclosed double quote",
                self.line, self.pos
            ));
        }
        return Ok(Token {
            kind: TokenKind::FieldValue,
            data: value_data,
        });
    }

    /// https://man7.org/linux/man-pages/man5/ssh_config.5.html
    pub fn tokenize(mut self) -> Result<Vec<Token>, String> {
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
}
