use std::fmt;
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum LineEnding {
    Lf,
    Crlf
}

impl fmt::Display for LineEnding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LineEnding::Lf => write!(f, "\n"),
            LineEnding::Crlf => write!(f, "\r\n")
        }
    }
}

#[derive(Default)]
pub(crate) struct Decor {
    indent: Option<String>,
    ending: Option<LineEnding>,
}

impl Decor {
    pub(crate) fn indent(&self) -> Option<&str> {
        self.indent.as_deref()
    }

    pub(crate) fn set_indent(&mut self, indent: &str) -> Result<()> {
        if indent.chars().any(|c| !is_inline_ws(c)) || indent.is_empty() {
            return Err(Error::InvalidIndent(indent.into()));
        }
        self.indent = Some(indent.into());
        Ok(())
    }

    pub(crate) fn set_indent_if_absent(&mut self, indent: &str) -> Result<()> {
        if self.indent.is_none() {
            self.set_indent(indent)?;
        }
        Ok(())
    }

    pub(crate) fn with_indent(mut self, indent: &str) -> Result<Self> {
        self.set_indent(indent)?;
        Ok(self)
    }

    pub(crate) fn write_indent(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indent().unwrap_or(""))
    }

    pub(crate) fn ending(&self) -> Option<&str> {
        self.ending.as_deref()
    }

    pub(crate) fn set_ending(&mut self, ending: &str) -> Result<()> {
        if ending != "\n" && ending != "\r\n" {
            return Err(Error::InvalidLineEnding(ending.into()));
        }
        self.ending = Some(ending.into());
        Ok(())
    }

    pub(crate) fn set_ending_if_absent(&mut self, ending: &str) -> Result<()> {
        if self.ending.is_none() {
            self.set_ending(ending)?;
        }
        Ok(())
    }

    pub(crate) fn with_ending(mut self, ending: &str) -> Result<Self> {
        self.set_ending(ending)?;
        Ok(self)
    }

    pub(crate) fn write_ending(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ending().unwrap_or(""))
    }
}