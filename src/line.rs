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
        if indent.chars().any(|c| !is_inline_ws(c)) || indent.is_empty() {
            return Err(
                "unexpected indent content: indent should be composed of whitespace only".into(),
            );
        }
        self.indent = Some(Token {
            kind: TokenKind::WhiteSpace,
            data: indent.to_string(),
        });
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
        if ending != "\n" && ending != "\r\n" {
            return Err("unexpected ending content: ending should be '\\n' or '\\r\\n'".into());
        }
        self.ending = Some(Token {
            kind: TokenKind::LineEnding,
            data: ending.to_string(),
        });
        Ok(self)
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
        return Ok(lines);
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
                return Ok(Line::Directive(Directive {
                    indent,
                    key,
                    sep,
                    value,
                    ending,
                }));
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
