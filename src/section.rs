use std::fmt;
use crate::line::{Directive, Line};
use crate::field_keys::FieldKey;

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

impl Section {
    pub fn parse_sections(lines: Vec<Line>) -> (Vec<Line>, Vec<Section>) {
        let mut preamble: Vec<Line> = Vec::new();
        let mut sections: Vec<Section> = Vec::new();

        for line in lines {
            match line {
                Line::Directive(d) if FieldKey::parse(&d.key.data).is_selector() => {
                    sections.push(Section { header: d, body: Vec::new() });
                }
                line => match sections.last_mut() {
                    Some(section) => section.body.push(line),
                    None => preamble.push(line),
                }
            }
        }
        (preamble, sections)
    }
}