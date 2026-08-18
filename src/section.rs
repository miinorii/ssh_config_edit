use crate::decor::{Decor, Indent, LineEnding};
use crate::error::{Error, Result};
use crate::field_keys::FieldKey;
use crate::line::{Directive, Line, LineKind, Selector};
use std::fmt;

fn valid_newline(line: &Line) -> Result<()> {
    match line.kind() {
        LineKind::Directive(d) if d.is_selector() => {
            Err(Error::UnexpectedSelector(line.to_string()))
        }
        _ => Ok(()),
    }
}

pub struct Section {
    header: Selector,
    body: Vec<Line>,
    default_decor: Decor,
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
        let decor = Decor::default();

        Self {
            header,
            body: Vec::new(),
            default_decor: decor,
        }
    }

    /// Set default indent.
    ///
    /// The default is a fallback used only when the section has
    /// no line to infer from, it never rewrites existing lines.
    pub fn set_default_indent(&mut self, indent: Indent) {
        self.default_decor.set_indent(indent)
    }

    /// Returns `Self` with the default indent set to `indent`.
    pub fn with_indent(mut self, indent: Indent) -> Self {
        self.set_default_indent(indent);
        self
    }

    /// Set default ending.
    ///
    /// The default is a fallback used only when the section has
    /// no line to infer from, it never rewrites existing lines.
    pub fn set_default_ending(&mut self, ending: LineEnding) {
        self.default_decor.set_ending(ending)
    }

    /// Returns `Self` with the default ending set to `ending`.
    pub fn with_ending(mut self, ending: LineEnding) -> Self {
        self.set_default_ending(ending);
        self
    }

    /// Returns the current header as a [`Selector`].
    pub fn header(&self) -> &Selector {
        &self.header
    }

    /// Returns the current header as a mutable [`Selector`].
    pub fn header_mut(&mut self) -> &mut Selector {
        &mut self.header
    }

    /// Returns an iterator over all [`Line`].
    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.body.iter()
    }

    /// Returns an iterator that allows modifying all [`Line`].
    pub fn lines_mut(&mut self) -> impl Iterator<Item = &mut Line> {
        self.body.iter_mut()
    }

    /// Returns an iterator over all [`Directive`].
    pub fn directives(&self) -> impl Iterator<Item = &Directive> {
        self.lines().filter_map(Line::as_directive)
    }

    /// Returns an iterator that allows modifying all [`Directive`].
    pub fn directives_mut(&mut self) -> impl Iterator<Item = &mut Directive> {
        self.lines_mut().filter_map(Line::as_directive_mut)
    }

    /// Returns the first [`Directive`] matching `key`.
    pub fn get_one(&self, key: &FieldKey) -> Option<&Directive> {
        self.directives().find(|d| d.field_key() == *key)
    }

    /// Returns the first [`Directive`] matching `key` with mutability.
    pub fn get_one_mut(&mut self, key: &FieldKey) -> Option<&mut Directive> {
        self.directives_mut().find(|d| d.field_key() == *key)
    }

    /// Returns all [`Directive`] matching `key`.
    pub fn get_all(&self, key: &FieldKey) -> impl Iterator<Item = &Directive> {
        self.directives().filter(|d| d.field_key() == *key)
    }

    /// Returns all [`Directive`] matching `key` with mutability.
    pub fn get_all_mut(&mut self, key: &FieldKey) -> impl Iterator<Item = &mut Directive> {
        self.directives_mut().filter(|d| d.field_key() == *key)
    }

    /// Append `line` and add a line terminator to the previous header/line if none is set.
    pub fn push(&mut self, line: Line) -> Result<()> {
        self.insert(self.line_count(), line)
    }

    /// Insert `line` at `index` and add a line terminator to the previous header/line if none is set.
    /// # Panics
    /// Panics if `index` is greater than [`Self::line_count`].
    pub fn insert(&mut self, index: usize, mut line: Line) -> Result<()> {
        valid_newline(&line)?;
        let line_ending = self.infer_line_ending();
        let line_indent = self.infer_line_indent();

        self.terminate(line_ending);

        // avoid useless indent on newly created blank lines
        if !line.is_blank() {
            line.set_indent_if_absent(line_indent);
        }
        line.set_ending_if_absent(line_ending);

        self.body.insert(index, line);
        Ok(())
    }

    /// Remove the [`Line`] at `index`.
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove(&mut self, index: usize) -> Line {
        self.body.remove(index)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns false.
    /// This method operates in place, visiting each element exactly once in the original order, and preserves the order of the retained elements.
    pub fn retain(&mut self, f: impl FnMut(&Line) -> bool) {
        self.body.retain(f);
    }

    /// Returns the [`Line`] at `index`.
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn line(&self, index: usize) -> &Line {
        &self.body[index]
    }

    /// Returns the [`Line`] at `index` with mutability.
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn line_mut(&mut self, index: usize) -> &mut Line {
        &mut self.body[index]
    }

    /// Returns the [`Line`] count.
    pub fn line_count(&self) -> usize {
        self.body.len()
    }

    pub fn indent(&self) -> Option<&Indent> {
        self.header
            .indent()
            .or_else(|| self.body.iter().find_map(Line::indent))
    }

    pub fn ending(&self) -> Option<LineEnding> {
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
            section.set_default_indent(section.infer_line_indent());
            section.set_default_ending(section.infer_line_ending());
        }
        Ok((preamble, sections))
    }

    pub(crate) fn infer_line_ending(&self) -> LineEnding {
        self.ending()
            .or_else(|| self.default_decor.ending())
            .unwrap_or_default()
    }

    pub(crate) fn infer_line_indent(&self) -> Indent {
        self.indent()
            .or_else(|| self.default_decor.indent())
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn terminate(&mut self, ending: LineEnding) {
        self.header.set_ending_if_absent(ending);
        if let Some(last_line) = self.body.last_mut() {
            last_line.set_ending_if_absent(ending);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(data: &str) -> (Vec<Line>, Vec<Section>) {
        let lines = Line::parse_lines(Lexer::new(data).tokenize().unwrap());
        Section::parse_sections(lines).unwrap()
    }

    fn section_from(data: &str) -> Section {
        let (_, mut sections) = parse(data);
        sections.remove(0)
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
        let line = Line::directive(FieldKey::User, "x")
            .unwrap()
            .with_indent(Indent::default());
        s.push(line).unwrap();

        let ending = LineEnding::default();
        assert_eq!(s.to_string(), format!("Host a{ending}\tUser x{ending}",));
    }

    #[test]
    fn push_line_terminates_unterminated_last_body_line() {
        let mut s = section_from("Host a\n\tUser x");
        let line = Line::directive(FieldKey::Hostname, "1.2.3.4")
            .unwrap()
            .with_indent(Indent::default());
        s.push(line).unwrap();
        assert_eq!(s.to_string(), "Host a\n\tUser x\n\tHostname 1.2.3.4\n");
    }
}
