use crate::lexer::{Token, TokenKind};
use std::{fmt, iter::Peekable, vec};

pub struct Directive {
    pub indent: Option<Token>,
    pub key: Token,
    pub sep: Token,
    pub value: Token,
    pub ending: Option<Token>,
}

impl Directive {
    pub fn new(key: &str, value: &str) -> Result<Self, String> {
        if key.is_empty() {
            return Err("key is empty".into());
        }

        if value.is_empty() {
            return Err("value is empty".into());
        }

        Ok(Self {
            indent: None,
            key: Token {
                kind: TokenKind::FieldKey,
                data: key.into(),
            },
            sep: Token {
                kind: TokenKind::FieldSeparator,
                data: " ".into(),
            },
            value: Token {
                kind: TokenKind::FieldValue,
                data: value.into(),
            },
            ending: None,
        })
    }

    pub fn with_indent(mut self, indent: &str) -> Result<Self, String> {
        self.set_indent(indent)?;
        Ok(self)
    }

    pub fn with_sep(mut self, sep: &str) -> Result<Self, String> {
        if sep.chars().any(|c| !is_inline_ws(c) && c != '=')
            || sep.chars().filter(|c| *c == '=').count() > 1
            || sep.is_empty()
        {
            return Err("unexpected separator content: separator should be composed of whitespaces and at most one '='".into());
        }
        self.sep.data = sep.into();
        Ok(self)
    }

    pub fn with_ending(mut self, ending: &str) -> Result<Self, String> {
        self.set_ending(ending)?;
        Ok(self)
    }

    pub fn set_ending(&mut self, ending: &str) -> Result<(), String> {
        self.ending = Some(ending_token(ending)?);
        Ok(())
    }

    pub fn set_indent(&mut self, indent: &str) -> Result<(), String> {
        self.indent = Some(indent_token(indent)?);
        Ok(())
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(indent) = &self.indent {
            write!(f, "{indent}")?;
        }
        write!(f, "{}{}{}", &self.key, &self.sep, &self.value)?;
        if let Some(ending) = &self.ending {
            write!(f, "{ending}")?;
        }
        Ok(())
    }
}

pub enum Line {
    Directive(Directive),
    Comment {
        indent: Option<Token>,
        text: Token,
        ending: Option<Token>,
    },
    Blank {
        indent: Option<Token>,
        ending: Option<Token>,
    },
}

impl Line {
    /// Parse multiple `Line` from a `Vec<Token>`.
    pub fn parse_lines(tokens: Vec<Token>) -> Result<Vec<Self>, String> {
        let mut iter = tokens.into_iter().peekable();
        let mut lines: Vec<Self> = Vec::new();
        while iter.peek().is_some() {
            lines.push(Self::parse_line(&mut iter)?);
        }
        Ok(lines)
    }

    pub fn indent(&self) -> Option<&Token> {
        match self {
            Line::Directive(d) => d.indent.as_ref(),
            Line::Comment { indent, .. } | Line::Blank { indent, .. } => indent.as_ref(),
        }
    }

    pub fn ending(&self) -> Option<&Token> {
        match self {
            Line::Directive(d) => d.ending.as_ref(),
            Line::Comment { ending, .. } | Line::Blank { ending, .. } => ending.as_ref(),
        }
    }

    pub fn set_ending(&mut self, ending: &str) -> Result<(), String> {
        match self {
            Line::Directive(d) => d.set_ending(ending)?,
            Line::Comment { ending: e, .. } | Line::Blank { ending: e, .. } => {
                let token = ending_token(ending)?;
                *e = Some(token);
            }
        }
        Ok(())
    }

    pub fn set_indent(&mut self, indent: &str) -> Result<(), String> {
        match self {
            Line::Directive(d) => d.set_indent(indent)?,
            Line::Comment { indent: e, .. } | Line::Blank { indent: e, .. } => {
                let token = indent_token(indent)?;
                *e = Some(token);
            }
        }
        Ok(())
    }

    /// Parse the next line from the token stream.
    ///
    /// Assume each line can either be one of the following pattern:
    ///
    /// - `[indent], comment, [line_ending]`
    /// - `[indent], key, separator, value, [line_ending]`
    /// - `indent, [line_ending]`
    /// - `line_ending`
    ///
    /// Optionnal token are denoted with `[]`
    fn parse_line(iter: &mut Peekable<vec::IntoIter<Token>>) -> Result<Line, String> {
        let indent = iter.next_if(|t| t.kind == TokenKind::WhiteSpace);
        match iter.peek().map(|t| &t.kind) {
            Some(TokenKind::Comment) => Ok(Line::Comment {
                indent,
                text: iter.next().unwrap(),
                ending: iter.next_if(|t| t.kind == TokenKind::LineEnding),
            }),
            Some(TokenKind::LineEnding) | None => Ok(Line::Blank {
                indent,
                ending: iter.next(), // None if EOF
            }),
            Some(TokenKind::FieldKey) => {
                let key = iter.next().unwrap();
                let sep = iter
                    .next_if(|t| t.kind == TokenKind::FieldSeparator)
                    .ok_or("expected FieldSeparator")?;
                let value = iter
                    .next_if(|t| t.kind == TokenKind::FieldValue)
                    .ok_or("expected FieldValue")?;
                let ending = iter.next_if(|t| t.kind == TokenKind::LineEnding);
                Ok(Line::Directive(Directive {
                    indent,
                    key,
                    sep,
                    value,
                    ending,
                }))
            }
            Some(other) => Err(format!("unexpected token: {other:?}")),
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Directive(d) => {
                write!(f, "{d}")?;
            }
            Line::Comment {
                indent,
                text,
                ending,
            } => {
                if let Some(indent) = indent {
                    write!(f, "{indent}")?;
                }
                write!(f, "{text}")?;
                if let Some(ending) = ending {
                    write!(f, "{ending}")?;
                }
            }
            Line::Blank { indent, ending } => {
                if let Some(indent) = indent {
                    write!(f, "{indent}")?;
                }
                if let Some(ending) = ending {
                    write!(f, "{ending}")?;
                }
            }
        }
        Ok(())
    }
}

fn is_inline_ws(c: char) -> bool {
    c.is_whitespace() && c != '\n' && c != '\r'
}

fn ending_token(ending: &str) -> Result<Token, String> {
    if ending != "\n" && ending != "\r\n" {
        return Err("unexpected ending content: ending should be '\\n' or '\\r\\n'".into());
    }
    Ok(Token {
        kind: TokenKind::LineEnding,
        data: ending.to_string(),
    })
}

fn indent_token(indent: &str) -> Result<Token, String> {
    if indent.chars().any(|c| !is_inline_ws(c)) || indent.is_empty() {
        return Err(
            "unexpected indent content: indent should be composed of whitespace only".into(),
        );
    }
    Ok(Token {
        kind: TokenKind::WhiteSpace,
        data: indent.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn roundtrip(data: &str) -> String {
        let lines = Line::parse_lines(Lexer::new(data).tokenize().unwrap()).unwrap();
        lines.iter().map(|l| l.to_string()).collect()
    }

    // --- roundtrip tests ---

    #[test]
    fn roundtrip_mixed_content() {
        let data = "# global\nAddKeysToAgent yes\n\nHost a\n\tHostName 1.2.3.4\n";
        assert_eq!(roundtrip(data), data);
    }

    #[test]
    fn roundtrip_crlf() {
        let data = "Host a\r\n\tUser x\r\n";
        assert_eq!(roundtrip(data), data);
    }

    #[test]
    fn roundtrip_no_trailing_newline() {
        let data = "Host a\n\tUser x";
        assert_eq!(roundtrip(data), data);
    }

    #[test]
    fn roundtrip_equals_separator() {
        let data = "Host = a\n\tPort=22\n";
        assert_eq!(roundtrip(data), data);
    }

    #[test]
    fn roundtrip_trailing_blank_lines() {
        let data = "Host a\n\tUser x\n\n   ";
        assert_eq!(roundtrip(data), data);
    }

    // --- Directive tests ---

    fn sample_directive() -> Directive {
        Directive::new("User", "x").unwrap()
    }

    #[test]
    fn with_ending_rejects_garbage() {
        assert!(sample_directive().with_ending("\r").is_err());
        assert!(sample_directive().with_ending(" \n").is_err());
    }

    #[test]
    fn with_sep_accepts_valid_forms() {
        assert!(sample_directive().with_sep(" ").is_ok());
        assert!(sample_directive().with_sep("=").is_ok());
        assert_eq!(sample_directive().with_sep(" = ").unwrap().sep.data, " = ");
    }

    #[test]
    fn with_sep_rejects_double_equal() {
        assert!(sample_directive().with_sep("==").is_err());
        assert!(sample_directive().with_sep("a==").is_err());
    }

    #[test]
    fn with_sep_rejects_newline() {
        assert!(sample_directive().with_sep(" \n ").is_err());
    }

    #[test]
    fn with_sep_rejects_empty() {
        assert!(sample_directive().with_sep("").is_err());
    }

    #[test]
    fn with_indent_rejects_newline() {
        assert!(sample_directive().with_indent("\n").is_err());
    }

    #[test]
    fn with_indent_accepts_blank_chars() {
        assert!(sample_directive().with_indent("\t  ").is_ok());
    }
}
