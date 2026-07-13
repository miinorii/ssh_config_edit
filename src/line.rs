use std::{iter::Peekable, vec, fmt};
use crate::lexer::{Token, TokenKind};

pub struct Directive {
    pub indent: Option<Token>,
    pub key: Token,
    pub sep: Token,
    pub value: Token,
    pub ending: Option<Token>
}

pub enum Line {
    Directive(Directive),
    Comment { indent: Option<Token>, text: Token, ending: Option<Token>},
    Blank { indent: Option<Token>, ending: Option<Token>},
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
                ending: iter.next_if(|t| t.kind == TokenKind::LineEnding) 
            }),
            Some(TokenKind::LineEnding) | None => Ok(Line::Blank { 
                indent, 
                ending: iter.next() // None if EOF 
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
                return Ok(Line::Directive(Directive{ 
                    indent, 
                    key, 
                    sep, 
                    value, 
                    ending 
                }));
            },
            Some(other) => Err(format!("unexpected token: {other:?}"))
        }
    }
}


impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Directive(d) => {
                if let Some(indent) = &d.indent {
                    write!(f, "{indent}")?;
                }
                write!(f, "{}{}{}",d.key, d.sep, d.value)?;
                if let Some(ending) = &d.ending {
                    write!(f, "{ending}")?;
                }
            },
            Line::Comment { indent, text, ending } => {
                if let Some(indent) = indent {
                    write!(f, "{indent}")?;
                }
                write!(f, "{text}")?;
                if let Some(ending) = ending {
                    write!(f, "{ending}")?;
                }
            },
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