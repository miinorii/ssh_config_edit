use crate::error::{Error, Result};
use crate::field_keys::FieldKey;
use crate::lexer::LexItem;
use std::{fmt, iter::Peekable, vec};

fn is_inline_ws(c: char) -> bool {
    c.is_whitespace() && c != '\n' && c != '\r'
}

#[derive(Default)]
struct Decor {
    indent: Option<String>,
    ending: Option<String>,
}

impl Decor {
    fn indent(&self) -> Option<&str> {
        self.indent.as_deref()
    }

    fn set_indent(&mut self, indent: &str) -> Result<()> {
        if indent.chars().any(|c| !is_inline_ws(c)) || indent.is_empty() {
            return Err(Error::InvalidIndent(indent.into()));
        }
        self.indent.insert(indent.into());
        Ok(())
    }

    fn set_indent_if_absent(&mut self, indent: &str) -> Result<()> {
        if self.indent.is_none() {
            self.set_indent(indent)?;
        }
        Ok(())
    }

    fn with_indent(mut self, indent: &str) -> Result<Self> {
        self.set_indent(indent)?;
        Ok(self)
    }

    pub(crate) fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indent().unwrap_or(""))
    }

    fn ending(&self) -> Option<&str> {
        self.ending.as_deref()
    }

    fn set_ending(&mut self, ending: &str) -> Result<()> {
        if ending != "\n" && ending != "\r\n" {
            return Err(Error::InvalidLineEnding(ending.into()));
        }
        self.ending.insert(ending.into());
        Ok(())
    }

    fn set_ending_if_absent(&mut self, ending: &str) -> Result<()> {
        if self.ending.is_none() {
            self.set_ending(ending)?;
        }
        Ok(())
    }

    fn with_ending(mut self, ending: &str) -> Result<Self> {
        self.set_ending(ending)?;
        Ok(self)
    }

    pub(crate) fn write_ending(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ending().unwrap_or(""))
    }
}

pub struct Directive {
    key: String,
    sep: String,
    value: String,
}

impl Directive {
    pub fn new(key: &str, value: &str) -> Result<Self> {
        if key.is_empty() {
            return Err(Error::EmptyKey);
        }

        if value.is_empty() {
            return Err(Error::EmptyValue);
        }

        Ok(Self {
            key: key.into(),
            sep: " ".into(),
            value: value.into(),
        })
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn field_key(&self) -> FieldKey {
        FieldKey::parse(&self.key)
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) -> Result<()> {
        if value.len() == 0 || value.chars().all(char::is_whitespace) {
            return Err(Error::EmptyValue);
        }
        self.value = value.into();
        Ok(())
    }

    pub fn separator(&self) -> &str {
        &self.sep
    }

    pub fn with_sep(mut self, sep: &str) -> Result<Self> {
        if sep.chars().any(|c| !is_inline_ws(c) && c != '=')
            || sep.chars().filter(|c| *c == '=').count() > 1
            || sep.is_empty()
        {
            return Err(Error::InvalidSeparator(sep.into()));
        }
        self.sep = sep.into();
        Ok(self)
    }

    pub fn is_cumulative(&self) -> bool {
        FieldKey::parse(&self.key).is_cumulative()
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", &self.key, &self.sep, &self.value)?;
        Ok(())
    }
}

struct Selector {
    decor: Decor,
    sep: String,
    keyword: String,
    value: String,
}

pub struct Line {
    decor: Decor,
    kind: LineKind,
}

enum LineKind {
    Directive(Directive),
    Comment(String),
    Blank,
}

impl Line {
    pub fn indent(&self) -> Option<&str> {
        self.decor.indent()
    }

    pub fn set_indent(&mut self, indent: &str) -> Result<()> {
        self.decor.set_indent(indent)
    }

    pub fn set_indent_if_absent(&mut self, indent: &str) -> Result<()> {
        self.decor.set_indent_if_absent(indent)
    }

    pub fn with_indent(mut self, indent: &str) -> Result<Self> {
        self.decor.set_indent(indent)?;
        Ok(self)
    }

    pub fn ending(&self) -> Option<&str> {
        self.decor.ending()
    }

    pub fn set_ending(&mut self, ending: &str) -> Result<()> {
        self.decor.set_ending(ending)
    }

    pub fn set_ending_if_absent(&mut self, ending: &str) -> Result<()> {
        self.decor.set_ending_if_absent(ending)
    }

    pub fn with_ending(mut self, ending: &str) -> Result<Self> {
        self.decor.set_ending(ending)?;
        Ok(self)
    }

    /// Parse multiple `Line` from a `Vec<Token>`.
    pub(crate) fn parse_lines(items: Vec<LexItem>) -> Result<Vec<Self>> {
        let mut iter = items.into_iter().peekable();
        let mut lines: Vec<Self> = Vec::new();
        while iter.peek().is_some() {
            lines.push(Self::parse_line(&mut iter)?);
        }
        Ok(lines)
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
    fn parse_line(iter: &mut Peekable<vec::IntoIter<LexItem>>) -> Result<Line> {
        let indent = iter
            .next_if(|i| matches!(i, LexItem::Indent(_)))
            .and_then(LexItem::into_indent);

        // an Ending here means the line has no content
        let kind = match iter.next_if(|i| !matches!(i, LexItem::Ending(_))) {
            Some(LexItem::Comment(text)) => LineKind::Comment(text),
            Some(LexItem::Directive { key, sep, value }) => {
                LineKind::Directive(Directive::new(&key, &value)?.with_sep(&sep)?)
            }
            Some(LexItem::Indent(_)) => unreachable!("indent already checked"),
            Some(LexItem::Ending(_)) => unreachable!("excluded by next_if"),
            None => LineKind::Blank,
        };

        let ending = iter
            .next_if(|i| matches!(i, LexItem::Ending(_)))
            .and_then(LexItem::into_ending);

        Ok(Line {
            decor: Decor { indent, ending },
            kind,
        })
    }

    pub fn as_comment(&self) -> Option<&str> {
        match &self.kind {
            LineKind::Comment(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_directive(&self) -> Option<&Directive> {
        match &self.kind {
            LineKind::Directive(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_directive_mut(&mut self) -> Option<&mut Directive> {
        match &mut self.kind {
            LineKind::Directive(d) => Some(d),
            _ => None,
        }
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(indent) = self.decor.indent() {
            write!(f, "{indent}")?;
        }

        match &self.kind {
            LineKind::Directive(d) => write!(f, "{d}")?,
            LineKind::Comment(text) => write!(f, "{text}")?,
            LineKind::Blank => {} //no-op
        }

        if let Some(ending) = self.decor.ending() {
            write!(f, "{ending}")?;
        }
        Ok(())
    }
}

pub trait LineIterExt<'a> {
    fn any_directives(self) -> impl Iterator<Item = &'a Directive>;
    fn cumulative_directives(self) -> impl Iterator<Item = &'a Directive>;
}

impl<'a, I: Iterator<Item = &'a Line>> LineIterExt<'a> for I {
    fn any_directives(self) -> impl Iterator<Item = &'a Directive> {
        self.filter_map(|l| l.as_directive())
    }

    fn cumulative_directives(self) -> impl Iterator<Item = &'a Directive> {
        self.filter_map(|l| l.as_directive())
            .filter(|d| d.is_cumulative())
    }
}

pub trait LineIterMutExt<'a> {
    fn any_directives_mut(self) -> impl Iterator<Item = &'a mut Directive>;
    fn cumulative_directives_mut(self) -> impl Iterator<Item = &'a mut Directive>;
}
impl<'a, I: Iterator<Item = &'a mut Line>> LineIterMutExt<'a> for I {
    fn any_directives_mut(self) -> impl Iterator<Item = &'a mut Directive> {
        self.filter_map(|l| l.as_directive_mut())
    }

    fn cumulative_directives_mut(self) -> impl Iterator<Item = &'a mut Directive> {
        self.filter_map(|l| l.as_directive_mut())
            .filter(|d| d.is_cumulative())
    }
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

    // --- Line tests ---

    fn sample_directive() -> Directive {
        Directive::new("User", "x").unwrap()
    }

    fn sample_line() -> Line {
        Line {
            decor: Decor::default(),
            kind: LineKind::Directive(sample_directive()),
        }
    }

    #[test]
    fn with_ending_rejects_garbage() {
        assert!(sample_line().with_ending("\r").is_err());
        assert!(sample_line().with_ending(" \n").is_err());
    }

    #[test]
    fn with_indent_rejects_newline() {
        assert!(sample_line().with_indent("\n").is_err());
    }

    #[test]
    fn with_indent_accepts_blank_chars() {
        assert!(sample_line().with_indent("\t  ").is_ok());
    }

    #[test]
    fn with_sep_accepts_valid_forms() {
        assert!(sample_directive().with_sep(" ").is_ok());
        assert!(sample_directive().with_sep("=").is_ok());
        assert_eq!(
            sample_directive().with_sep(" = ").unwrap().separator(),
            " = "
        );
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
}
