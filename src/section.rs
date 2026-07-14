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
        self.header
            .indent
            .as_ref()
            .or_else(|| self.body.iter().find_map(Line::indent))
    }

    pub fn ending(&self) -> Option<&Token> {
        self.header.ending.as_ref()
    }

    /// Append `line` and add a line terminator to the previous header/line if none is set.
    pub fn push_line(&mut self, line: Line, ending: &str) -> Result<(), String> {
        if self.header.ending.is_none() {
            self.header.set_ending(ending)?;
        }

        if let Some(last_line) = self.body.last_mut()
            && last_line.ending().is_none()
        {
            last_line.set_ending(ending)?;
        }
        self.body.push(line);
        Ok(())
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

    fn section_from(data: &str) -> Section {
        let (_, mut sections) = parse(data);
        sections.remove(0)
    }

    fn field_line(key: &str, value: &str, ending: &str) -> Line {
        Line::Directive(
            Directive::new(key, value)
                .unwrap()
                .with_indent("\t")
                .unwrap()
                .with_ending(ending)
                .unwrap(),
        )
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

    #[test]
    fn push_line_terminates_unterminated_header() {
        let mut s = section_from("Host a");
        s.push_line(field_line("User", "x", "\n"), "\n").unwrap();
        assert_eq!(s.to_string(), "Host a\n\tUser x\n");
    }

    #[test]
    fn push_line_terminates_unterminated_last_body_line() {
        let mut s = section_from("Host a\n\tUser x");
        s.push_line(field_line("Hostname", "1.2.3.4", "\n"), "\n")
            .unwrap();
        assert_eq!(s.to_string(), "Host a\n\tUser x\n\tHostname 1.2.3.4\n");
    }
}
