use crate::decor::{Decor, Indent, LineEnding, is_inline_ws};
use crate::error::{Error, Result};
use crate::field_keys::{FieldKey, SelectorKind};
use crate::lexer::LexItem;
use std::{fmt, iter::Peekable, vec};

fn validate_sep(sep: &str) -> Result<()> {
    if sep.chars().any(|c| !is_inline_ws(c) && c != '=')
        || sep.chars().filter(|c| *c == '=').count() > 1
        || sep.is_empty()
    {
        return Err(Error::InvalidSeparator(sep.into()));
    }
    Ok(())
}

fn validate_value(value: &str) -> Result<()> {
    if value.chars().all(char::is_whitespace) {
        return Err(Error::EmptyValue);
    }

    if value.contains(['\n', '\r']) {
        return Err(Error::InvalidValue(value.into()));
    }
    Ok(())
}

#[derive(PartialEq)]
pub struct Directive {
    key: String,
    sep: String,
    value: String,
}

impl Directive {
    /// Returns a new [`Directive`] from a given `key` and `value`.
    ///
    /// [`Directive`] represents individual config fields
    /// without indent or line ending:
    /// ```text
    /// Host dev
    ///     User test <- User Directive
    /// Host local
    ///     Port 12345 <- Port Directive
    /// ```
    pub fn new(key: FieldKey, value: &str) -> Result<Self> {
        validate_value(value)?;

        Ok(Self {
            key: key.as_canonical_str().to_string(),
            sep: " ".into(),
            value: value.into(),
        })
    }

    /// Construct a new [`Directive`] by consuming a given `key`, `sep` and `value`.
    pub(crate) fn from_parts(key: String, sep: String, value: String) -> Self {
        Self { key, sep, value }
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
        validate_value(value)?;
        self.value = value.into();
        Ok(())
    }

    pub fn separator(&self) -> &str {
        &self.sep
    }

    pub fn set_separator(&mut self, sep: &str) -> Result<()> {
        validate_sep(sep)?;
        self.sep = sep.into();
        Ok(())
    }

    pub fn with_separator(mut self, sep: &str) -> Result<Self> {
        self.set_separator(sep)?;
        Ok(self)
    }

    /// Returns `true` if `key` is cumulative, see [`FieldKey::is_cumulative`].
    pub fn is_cumulative(&self) -> bool {
        self.field_key().is_cumulative()
    }

    /// Returns `true` if `key` is a selector, see [`FieldKey::is_selector`].
    pub fn is_selector(&self) -> bool {
        self.field_key().is_selector()
    }
}

impl fmt::Display for Directive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.key, self.sep, self.value)
    }
}

pub struct Selector {
    decor: Decor,
    key: String,
    sep: String,
    value: String,
}

impl Selector {
    /// Returns a new [`Selector`] from a given `key` and `value`.
    ///
    /// [`Selector`] represents section headers
    /// with indent and line ending:
    /// ```text
    /// Host dev <- Host selector
    ///     Port 12345
    /// Match user git <- Match selector
    ///     Port 12345
    /// ```
    pub fn new(key: FieldKey, value: &str) -> Result<Self> {
        key.as_selector_kind()
            .ok_or_else(|| Error::NotASelector(key.as_canonical_str().to_string()))?;
        validate_value(value)?;

        Ok(Self {
            decor: Decor::default(),
            key: key.as_canonical_str().to_string(),
            sep: " ".into(),
            value: value.into(),
        })
    }

    pub fn indent(&self) -> Option<&Indent> {
        self.decor.indent()
    }

    pub fn set_indent(&mut self, indent: Indent) {
        self.decor.set_indent(indent)
    }

    pub fn set_indent_if_absent(&mut self, indent: Indent) {
        self.decor.set_indent_if_absent(indent)
    }

    pub fn with_indent(mut self, indent: Indent) -> Self {
        self.decor.set_indent(indent);
        self
    }

    pub fn ending(&self) -> Option<LineEnding> {
        self.decor.ending()
    }

    pub fn set_ending(&mut self, ending: LineEnding) {
        self.decor.set_ending(ending);
    }

    pub fn set_ending_if_absent(&mut self, ending: LineEnding) {
        self.decor.set_ending_if_absent(ending);
    }

    pub fn with_ending(mut self, ending: LineEnding) -> Self {
        self.decor.set_ending(ending);
        self
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn field_key(&self) -> FieldKey {
        FieldKey::parse(&self.key)
    }

    pub fn kind(&self) -> SelectorKind {
        self.field_key()
            .as_selector_kind()
            .expect("validated in ::new")
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: &str) -> Result<()> {
        validate_value(value)?;
        self.value = value.into();
        Ok(())
    }

    pub fn separator(&self) -> &str {
        &self.sep
    }

    pub fn set_separator(&mut self, sep: &str) -> Result<()> {
        validate_sep(sep)?;
        self.sep = sep.into();
        Ok(())
    }

    pub fn with_separator(mut self, sep: &str) -> Result<Self> {
        self.set_separator(sep)?;
        Ok(self)
    }
}

impl TryFrom<Line> for Selector {
    type Error = Error;

    fn try_from(line: Line) -> Result<Self> {
        let Line { decor, kind } = line;
        match kind {
            LineKind::Directive(d) if d.is_selector() => Ok(Self {
                decor,
                key: d.key,
                sep: d.sep,
                value: d.value,
            }),
            other => Err(Error::NotASelector(Line { decor, kind: other }.to_string())),
        }
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decor.write_indent(f)?;
        write!(f, "{}{}{}", self.key, self.sep, self.value)?;
        self.decor.write_ending(f)
    }
}

#[derive(PartialEq)]
pub enum LineKind {
    Directive(Directive),
    Comment(String),
    Blank,
}

pub struct Line {
    decor: Decor,
    kind: LineKind,
}

impl From<Directive> for Line {
    fn from(directive: Directive) -> Self {
        Line {
            decor: Decor::default(),
            kind: LineKind::Directive(directive),
        }
    }
}

impl Line {
    /// Returns a new [`Line`] of [`LineKind::Directive`] from a given `key` and `value`.
    ///
    /// Such lines represent individual config fields
    /// with indent and line ending:
    /// ```text
    /// Host dev
    ///     User test <- User Directive
    /// Host local
    ///     Port 12345 <- Port Directive
    /// ```
    pub fn directive(key: FieldKey, value: &str) -> Result<Line> {
        Ok(Line::from(Directive::new(key, value)?))
    }

    /// Returns a new [`Line`] of [`LineKind::Comment`] from a given `text`.
    ///
    ///  `"# "` is prepended when `text` does not already start with `#`.
    ///
    /// Such lines represent comments:
    /// ```text
    /// # this is a comment <- LineKind::Comment
    /// Host 1.2.3.4
    ///     Port 1234
    /// ```
    ///
    /// # Errors
    /// Returns [`Error::InvalidComment`] if `text` contains `\n` or `\r`, since
    /// a comment spanning a line break would re-parse as two lines.
    pub fn comment(text: &str) -> Result<Line> {
        if text.contains(['\n', '\r']) {
            return Err(Error::InvalidComment(text.into()));
        }

        let text = if text.starts_with('#') {
            text.to_string()
        } else {
            format!("# {text}")
        };

        Ok(Self {
            decor: Decor::default(),
            kind: LineKind::Comment(text),
        })
    }

    /// Returns a new [`Line`] of [`LineKind::Blank`], on a blank line whitespace
    /// data is stored inside [`Self::indent`].
    ///
    /// Such lines represent empty lines:
    /// ```text
    /// Host 1.2.3.4
    ///     <- LineKind::Blank
    ///     Port 1234
    /// ```
    pub fn blank() -> Line {
        Self {
            decor: Decor::default(),
            kind: LineKind::Blank,
        }
    }

    /// Parse multiple [`Line`] from a `Vec<LexItem>`.
    pub(crate) fn parse_lines(items: Vec<LexItem>) -> Vec<Self> {
        let mut iter = items.into_iter().peekable();
        let mut lines: Vec<Self> = Vec::new();
        while iter.peek().is_some() {
            lines.push(Self::parse_line(&mut iter));
        }
        lines
    }

    /// Parse the next line from the LexItem stream.
    ///
    /// Assume each line can either be one of the following pattern:
    ///
    /// - `[Indent], Comment, [Ending]`
    /// - `[Indent], Directive, [Ending]`
    /// - `[Indent], [Ending]`
    ///
    /// Optional token are denoted with `[]`
    fn parse_line(iter: &mut Peekable<vec::IntoIter<LexItem>>) -> Line {
        let indent = iter
            .next_if(|i| matches!(i, LexItem::Indent(_)))
            .and_then(LexItem::into_indent);

        // an Ending here means the line has no content
        let kind = match iter.next_if(|i| !matches!(i, LexItem::Ending(_))) {
            Some(LexItem::Comment(text)) => LineKind::Comment(text),
            Some(LexItem::Directive { key, sep, value }) => {
                LineKind::Directive(Directive::from_parts(key, sep, value))
            }
            Some(LexItem::Indent(_)) => unreachable!("indent already checked"),
            Some(LexItem::Ending(_)) => unreachable!("excluded by next_if"),
            None => LineKind::Blank,
        };

        let ending = iter
            .next_if(|i| matches!(i, LexItem::Ending(_)))
            .and_then(LexItem::into_ending);

        let mut decor = Decor::new();
        if let Some(ending) = ending {
            decor.set_ending(ending);
        }
        if let Some(indent) = indent {
            decor.set_indent(Indent::from_lexer(indent));
        }
        Line { decor: decor, kind }
    }

    pub fn indent(&self) -> Option<&Indent> {
        self.decor.indent()
    }

    pub fn set_indent(&mut self, indent: Indent) {
        self.decor.set_indent(indent)
    }

    pub fn set_indent_if_absent(&mut self, indent: Indent) {
        self.decor.set_indent_if_absent(indent)
    }

    pub fn with_indent(mut self, indent: Indent) -> Self {
        self.decor.set_indent(indent);
        self
    }

    pub fn ending(&self) -> Option<LineEnding> {
        self.decor.ending()
    }

    pub fn set_ending(&mut self, ending: LineEnding) {
        self.decor.set_ending(ending);
    }

    pub fn set_ending_if_absent(&mut self, ending: LineEnding) {
        self.decor.set_ending_if_absent(ending);
    }

    pub fn with_ending(mut self, ending: LineEnding) -> Self {
        self.decor.set_ending(ending);
        self
    }

    pub fn kind(&self) -> &LineKind {
        &self.kind
    }

    /// Returns the [`LineKind::Comment`] text as `&str` and `None` otherwise.
    pub fn as_comment(&self) -> Option<&str> {
        match &self.kind {
            LineKind::Comment(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the [`LineKind::Comment`] text as `&mut String` and `None` otherwise.
    pub fn as_comment_mut(&mut self) -> Option<&mut String> {
        match &mut self.kind {
            LineKind::Comment(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the [`LineKind::Directive`] payload as [`&Directive`](Directive)
    /// and `None` otherwise.
    pub fn as_directive(&self) -> Option<&Directive> {
        match &self.kind {
            LineKind::Directive(d) => Some(d),
            _ => None,
        }
    }

    /// Returns the [`LineKind::Directive`] payload as [`&mut Directive`](Directive)
    /// and `None` otherwise.
    pub fn as_directive_mut(&mut self) -> Option<&mut Directive> {
        match &mut self.kind {
            LineKind::Directive(d) => Some(d),
            _ => None,
        }
    }

    /// Returns `true` if this is a [`LineKind::Blank`] and `false` otherwise.
    pub fn is_blank(&self) -> bool {
        self.kind == LineKind::Blank
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.decor.write_indent(f)?;
        match &self.kind {
            LineKind::Directive(d) => write!(f, "{d}")?,
            LineKind::Comment(text) => write!(f, "{text}")?,
            LineKind::Blank => {} //no-op
        }
        self.decor.write_ending(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn roundtrip(data: &str) -> String {
        let lines = Line::parse_lines(Lexer::new(data).tokenize().unwrap());
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
        Directive::new(FieldKey::User, "x").unwrap()
    }

    #[test]
    fn with_sep_accepts_valid_forms() {
        assert!(sample_directive().with_separator(" ").is_ok());
        assert!(sample_directive().with_separator("=").is_ok());
        assert_eq!(
            sample_directive()
                .with_separator(" = ")
                .unwrap()
                .separator(),
            " = "
        );
    }

    #[test]
    fn with_sep_rejects_double_equal() {
        assert!(sample_directive().with_separator("==").is_err());
        assert!(sample_directive().with_separator("a==").is_err());
    }

    #[test]
    fn with_sep_rejects_newline() {
        assert!(sample_directive().with_separator(" \n ").is_err());
    }

    #[test]
    fn with_sep_rejects_empty() {
        assert!(sample_directive().with_separator("").is_err());
    }
}
