use crate::field_keys::FieldKey;
use crate::lexer::Token;
use crate::line::{Directive, Line};
use std::fmt;


pub struct Section {
    pub header: Directive,
    pub body: Vec<Line>,
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
                    sections.push(Section {
                        header: d,
                        body: Vec::new(),
                    });
                }
                line => match sections.last_mut() {
                    Some(section) => section.body.push(line),
                    None => preamble.push(line),
                },
            }
        }
        (preamble, sections)
    }

    pub fn indent(&self) -> Option<&Token> {
        self.header.indent.as_ref().or_else(|| self.body.iter().find_map(Line::indent))
    }

    pub fn ending(&self) -> Option<&Token> {
        self.header.ending.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(data: &str) -> (Vec<Line>, Vec<Section>) {
        let lines = Line::parse_lines(Lexer::new(data).tokenize().unwrap()).unwrap();
        Section::parse_sections(lines)
    }

    #[test]
    fn preamble_collects_lines_before_first_section() {
        let (preamble, sections) = parse("# c\nAddKeysToAgent yes\n\nHost a\n\tUser x\n");
        assert_eq!(preamble.len(), 3); // comment, directive, blank
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header.value.data, "a");
        assert_eq!(sections[0].body.len(), 1);
    }

    #[test]
    fn match_starts_a_new_section() {
        let (_, sections) = parse("Host a\n\tUser x\nMatch user foo\n\tPort 22\n");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[1].header.key.data, "Match");
        assert_eq!(sections[1].body.len(), 1);
    }

    #[test]
    fn trailing_trivia_belongs_to_previous_section() {
        let data = "Host a\n\tUser x\n# note\n\n";
        let (preamble, sections) = parse(data);
        assert!(preamble.is_empty());
        assert_eq!(sections.len(), 1);
        let out: String = sections.iter().map(|s| s.to_string()).collect();
        assert_eq!(out, data); // Section::Display round-trips header + body
    }
}
