use std::fmt;
use crate::line::{Line, Directive};

pub struct Section {
    pub header: Directive,
    pub body: Vec<Line>
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.header)?;
        for line in &self.body {
            write!(f, "{line}")?;
        }
        Ok(())
    }
}