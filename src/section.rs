use crate::error::{Error, Result};
use crate::field_keys::FieldKey;
use crate::line::{Directive, Line, LineKind, Selector};
use std::fmt;

#[cfg(target_os = "windows")]
pub const DEFAULT_LINE_ENDING: &str = "\r\n";

#[cfg(not(target_os = "windows"))]
pub const DEFAULT_LINE_ENDING: &str = "\n";

pub const DEFAULT_LINE_INDENT: &str = "\t";

fn valid_newline(line: &Line) -> Result<()> {
    match line.kind() {
        LineKind::Directive(d) if d.is_selector() => {
            Err(Error::UnexpectedSelector(line.to_string()))
        }
        _ => Ok(())
    }
}

pub struct Section {
    header: Selector,
    body: Vec<Line>,
    default_indent: String,
    default_ending: String,
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
    pub fn new(header: Selector) -> Self {
        Self {
            header,
            body: Vec::new(),
            default_indent: DEFAULT_LINE_INDENT.into(),
            default_ending: DEFAULT_LINE_ENDING.into(),
        }
    }

    pub fn with_indent(mut self, indent: &str) -> Self {
        self.default_indent = indent.to_string();
        self
    }

    pub fn with_ending(mut self, ending: &str) -> Self {
        self.default_ending = ending.to_string();
        self
    }

    pub fn header(&self) -> &Selector {
        &self.header
    }

    pub fn header_mut(&mut self) -> &mut Selector {
        &mut self.header
    }

    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.body.iter()
    }

    pub fn lines_mut(&mut self) -> impl Iterator<Item = &mut Line> {
        self.body.iter_mut()
    }

    pub fn directives(&self) -> impl Iterator<Item = &Directive> {
        self.lines().filter_map(Line::as_directive)
    }

    pub fn directives_mut(&mut self) -> impl Iterator<Item = &mut Directive> {
        self.lines_mut().filter_map(Line::as_directive_mut)
    }

    pub fn get(&self, key: &FieldKey) -> Option<&Directive> {
        self.directives().find(|d| d.field_key() == *key)
    }

    pub fn get_mut(&mut self, key: &FieldKey) -> Option<&mut Directive> {
        self.directives_mut().find(|d| d.field_key() == *key)
    }

    pub fn get_all(&self, key: &FieldKey) -> impl Iterator<Item = &Directive> {
        self.directives().filter(|d| d.field_key() == *key)
    }

    pub fn get_all_mut(&mut self, key: &FieldKey) -> impl Iterator<Item = &mut Directive> {
        self.directives_mut().filter(|d| d.field_key() == *key)
    }

    /// Append `line` and add a line terminator to the previous header/line if none is set.
    pub fn push_line(&mut self, mut line: Line) -> Result<()> {
        valid_newline(&line)?;
        let line_ending = self.infer_line_ending();
        let line_indent = self.infer_line_indent();

        self.terminate(&line_ending)?;

        line.set_indent_if_absent(&line_indent)?;
        line.set_ending_if_absent(&line_ending)?;

        self.body.push(line);
        Ok(())
    }

    pub fn indent(&self) -> Option<&str> {
        self.header
            .indent()
            .or_else(|| self.body.iter().find_map(Line::indent))
    }

    pub fn ending(&self) -> Option<&str> {
        self.header
            .ending()
            .or_else(|| self.body.iter().find_map(Line::ending))
    }

    pub(crate) fn parse_sections(lines: Vec<Line>) -> Result<(Vec<Line>, Vec<Section>)> {
        let mut preamble: Vec<Line> = Vec::new();
        let mut sections: Vec<Section> = Vec::new();

        for line in lines {
            match line.kind() {
                LineKind::Directive(d) if d.field_key().is_selector() => {
                    let selector: Selector = line.try_into()?;
                    sections.push(Section::new(selector));
                }
                _ => match sections.last_mut() {
                    Some(section) => section.body.push(line),
                    None => preamble.push(line),
                },
            }
        }

        for section in sections.iter_mut() {
            section.default_ending = section.infer_line_ending().into();
            section.default_indent = section.infer_line_indent().into();
        }
        Ok((preamble, sections))
    }

    pub(crate) fn infer_line_ending(&self) -> String {
        self.ending()
            .map_or_else(|| self.default_ending.clone(), |t| t.to_string())
    }

    pub(crate) fn infer_line_indent(&self) -> String {
        self.indent()
            .map_or_else(|| self.default_indent.clone(), |t| t.to_string())
    }

    pub(crate) fn terminate(&mut self, ending: &str) -> Result<()> {
        self.header.set_ending_if_absent(ending)?;
        if let Some(last_line) = self.body.last_mut() {
            last_line.set_ending_if_absent(ending)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(data: &str) -> (Vec<Line>, Vec<Section>) {
        let lines = Line::parse_lines(Lexer::new(data).tokenize().unwrap()).unwrap();
        Section::parse_sections(lines).unwrap()
    }

    fn section_from(data: &str) -> Section {
        let (_, mut sections) = parse(data);
        sections.remove(0)
    }

    fn field_line(key: &str, value: &str) -> Line {
        Line::directive(key, value)
            .unwrap()
            .with_indent("\t")
            .unwrap()
    }

    #[test]
    fn preamble_collects_lines_before_first_section() {
        let (preamble, sections) = parse("# c\nAddKeysToAgent yes\n\nHost a\n\tUser x\n");
        assert_eq!(preamble.len(), 3); // comment, directive, blank
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].header.value(), "a");
        assert_eq!(sections[0].body.len(), 1);
    }

    #[test]
    fn match_starts_a_new_section() {
        let (_, sections) = parse("Host a\n\tUser x\nMatch user foo\n\tPort 22\n");
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[1].header.key(), "Match");
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
        s.push_line(field_line("User", "x")).unwrap();

        let ending = DEFAULT_LINE_ENDING;
        assert_eq!(s.to_string(), format!("Host a{ending}\tUser x{ending}",));
    }

    #[test]
    fn push_line_terminates_unterminated_last_body_line() {
        let mut s = section_from("Host a\n\tUser x");
        s.push_line(field_line("Hostname", "1.2.3.4")).unwrap();
        assert_eq!(s.to_string(), "Host a\n\tUser x\n\tHostname 1.2.3.4\n");
    }
}
