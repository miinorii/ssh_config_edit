use crate::error::{Error, Result};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum LineEnding {
    #[cfg_attr(not(target_os = "windows"), default)]
    Lf,
    #[cfg_attr(target_os = "windows", default)]
    Crlf,
}

impl LineEnding {
    pub fn as_str(&self) -> &str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

impl fmt::Display for LineEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Indent(String);

impl Indent {
    /// # Errors
    /// Returns [`Error::InvalidIndent`] unless `indent` is non-empty and contains
    /// only inline whitespace.
    pub fn new(indent: &str) -> Result<Self> {
        if indent.is_empty() || indent.chars().any(|c| !is_inline_ws(c)) {
            return Err(Error::InvalidIndent(indent.into()));
        }
        Ok(Self(indent.to_string()))
    }

    /// Caller need to guarantee that `indent` is non-empty inline whitespace.
    pub(crate) fn from_lexer(indent: String) -> Self {
        Self(indent)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Indent {
    fn default() -> Self {
        Self("\t".into())
    }
}

impl fmt::Display for Indent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Default)]
pub(crate) struct Decor {
    indent: Option<Indent>,
    ending: Option<LineEnding>,
}

impl Decor {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn indent(&self) -> Option<&Indent> {
        self.indent.as_ref()
    }

    pub(crate) fn set_indent(&mut self, indent: Indent) {
        self.indent = Some(indent);
    }

    pub(crate) fn set_indent_if_absent(&mut self, indent: Indent) {
        if self.indent.is_none() {
            self.set_indent(indent);
        }
    }

    pub(crate) fn with_indent(mut self, indent: Indent) -> Self {
        self.set_indent(indent);
        self
    }

    pub(crate) fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(indent) = self.indent() {
            write!(f, "{}", indent)?;
        }
        Ok(())
    }

    pub(crate) fn ending(&self) -> Option<LineEnding> {
        self.ending
    }

    pub(crate) fn set_ending(&mut self, ending: LineEnding) {
        self.ending = Some(ending);
    }

    pub(crate) fn set_ending_if_absent(&mut self, ending: LineEnding) {
        if self.ending.is_none() {
            self.set_ending(ending);
        }
    }

    pub(crate) fn with_ending(mut self, ending: LineEnding) -> Self {
        self.set_ending(ending);
        self
    }

    pub(crate) fn write_ending(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ending) = self.ending() {
            write!(f, "{}", ending)?;
        }
        Ok(())
    }
}

pub(crate) fn is_inline_ws(c: char) -> bool {
    c.is_whitespace() && c != '\n' && c != '\r'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_rejects_newline() {
        assert!(Indent::new("\n").is_err());
    }

    #[test]
    fn indent_accepts_blank_chars() {
        assert!(Indent::new("\t  ").is_ok());
    }
}
