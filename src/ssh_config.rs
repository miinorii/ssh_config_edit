use crate::decor::LineEnding;
use crate::error::Result;
use crate::field_keys::FieldKey;
use crate::lexer::Lexer;
use crate::line::{Line, LineKind, Selector};
use crate::section::Section;
use crate::settings::{Field, HostSettings};
use std::collections::HashMap;
use std::fmt;

pub struct SSHConfig {
    preamble: Vec<Line>,
    sections: Vec<Section>,
}

impl SSHConfig {
    /// Parse `data` into an editable document.
    ///
    /// The result round-trips byte for byte: `parse(data).to_string() == data`.
    ///
    /// # Errors
    /// Returns [`Error::Parse`](crate::Error::Parse) if `data` is not valid
    /// `ssh_config(5)` carrying the line and column of the offending character.
    pub fn parse(data: &str) -> Result<SSHConfig> {
        let lexer = Lexer::new(data);
        let lines = Line::parse_lines(lexer.tokenize()?);
        let (preamble, sections) = Section::parse_sections(lines)?;
        Ok(SSHConfig { preamble, sections })
    }

    /// Returns an iterator over every [`Section`] in document order.
    pub fn sections(&self) -> impl Iterator<Item = &Section> {
        self.sections.iter()
    }

    /// Returns an iterator that allows modifying every [`Section`],
    /// in document order.
    pub fn sections_mut(&mut self) -> impl Iterator<Item = &mut Section> {
        self.sections.iter_mut()
    }

    /// Returns the first [`Section`] whose header value is exactly `pattern`.
    ///
    /// The match is on the selector's raw text and is kind-agnostic, so
    /// `section("dev prod")` finds `Host dev prod` and `section("user git")`
    /// finds `Match user git`. No glob expansion happens here, see
    /// [`Self::resolve_host_settings`] for that.
    pub fn section(&self, pattern: &str) -> Option<&Section> {
        self.sections().find(|s| s.header().value() == pattern)
    }

    /// Returns the first [`Section`] whose header value is exactly `pattern`
    /// with mutability. See [`Self::section`] for the matching rules.
    pub fn section_mut(&mut self, pattern: &str) -> Option<&mut Section> {
        self.sections_mut().find(|s| s.header().value() == pattern)
    }

    /// Returns an iterator over the preamble, the [`Line`] appearing before the
    /// first selector.
    pub fn preamble(&self) -> impl Iterator<Item = &Line> {
        self.preamble.iter()
    }

    /// Returns an iterator that allows modifying the preamble, the [`Line`]
    /// appearing before the first selector.
    pub fn preamble_mut(&mut self) -> impl Iterator<Item = &mut Line> {
        self.preamble.iter_mut()
    }

    /// Insert `section` at `index`, terminating the preceding section or
    /// preamble line when it has no line ending of its own.
    ///
    /// `index == self.sections().count()` appends.
    ///
    /// # Panics
    /// Panics if `index` is greater than the number of sections.
    pub fn insert_section(&mut self, index: usize, section: Section) {
        // Ensure previous last line has a line ending
        if index == self.sections.len() {
            let ending = self.infer_line_ending();
            if let Some(prev) = self.sections.last_mut() {
                prev.terminate(ending);
            } else if let Some(last_line) = self.preamble.last_mut() {
                last_line.set_ending_if_absent(ending);
            }
        }

        self.sections.insert(index, section);
    }

    /// Remove and return the [`Section`] at `index`.
    ///
    /// # Panics
    /// Panics if `index` is out of bounds.
    pub fn remove_section(&mut self, index: usize) -> Section {
        self.sections.remove(index)
    }

    /// Append `section` to the end of the document.
    ///
    /// Terminates the preceding section or preamble line when it has no line
    /// ending of its own.
    pub fn push_section(&mut self, section: Section) {
        self.insert_section(self.sections.len(), section);
    }

    /// Remove and return the first [`Section`] whose header value is exactly
    /// `pattern`, or `None` if no section matches.
    ///
    /// See [`Self::section`] for the matching rules.
    pub fn remove_selector(&mut self, pattern: &str) -> Option<Section> {
        let index = self
            .sections
            .iter()
            .position(|s| s.header().value() == pattern)?;
        Some(self.sections.remove(index))
    }

    /// Infer line ending from the preamble and every [`Section`] header.
    ///
    /// Default to system default if no line ending is found.
    fn infer_line_ending(&self) -> LineEnding {
        self.preamble
            .iter()
            .find_map(Line::ending)
            .or_else(|| self.sections.iter().find_map(Section::ending))
            .unwrap_or_default()
    }

    /// Reconcile `host_settings` back into the document.
    ///
    /// If a section already matches [`HostSettings::host`] it is edited in
    /// place, otherwise a new one is inserted at the top. Editing is
    /// non-destructive: existing directives keep their separator and indent,
    /// comments and blank lines are preserved, and only the bytes of changed
    /// keys move.
    ///
    /// Directives whose key is absent from `host_settings` are removed.
    /// Cumulative keys keep the leading run of values that already match, then
    /// diverging values are dropped and the new ones appended in order.
    ///
    /// # Errors
    /// Returns [`Error::UnexpectedSelector`](crate::Error::UnexpectedSelector)
    /// if `host_settings` holds a selector key, and
    /// [`Error::EmptyValue`](crate::Error::EmptyValue) if a value is blank.
    pub fn set_host_settings(&mut self, host_settings: &HostSettings) -> Result<()> {
        let inferred_line_ending = self.infer_line_ending();
        let target_section = self.section_mut(host_settings.host());

        match target_section {
            Some(s) => {
                // ------ Cumulative key handling ------
                // Existing cumulative directives grouped by key -> (index, value).
                let mut existing: HashMap<FieldKey, Vec<(usize, String)>> = HashMap::new();
                for (i, line) in s.lines().enumerate() {
                    if let Some(d) = line.as_directive()
                        && d.is_cumulative()
                    {
                        existing
                            .entry(d.field_key())
                            .or_default()
                            .push((i, d.value().to_string()));
                    }
                }

                // Preserve cumulative keys until a divergence is observed (a new/different value).
                // When a divergence is observed start appending the fields
                // at the end of the current section.
                let mut to_remove: Vec<usize> = Vec::new();
                for key in host_settings.keys().filter(|k| k.is_cumulative()) {
                    let desired: Vec<&Field> = host_settings.get_all(key).collect();

                    let entries = existing.get(key).map(Vec::as_slice).unwrap_or_default();

                    // Keep the values that are identical to the new ones
                    // then stop at the first divergence.
                    let valid_count = entries
                        .iter()
                        .zip(&desired)
                        .take_while(|(current, wanted)| current.1 == wanted.value)
                        .count();

                    // From the first divergence drop the old lines, append the new values at the end.
                    // Order matters because `push` appends, so it cannot invalidate the indices collected here.
                    to_remove.extend(entries[valid_count..].iter().map(|(i, _)| *i));
                    for field in &desired[valid_count..] {
                        let line = Line::directive(key.clone(), &field.value)?;
                        s.push(line)?;
                    }
                }

                to_remove.sort();
                to_remove.reverse();
                for i in to_remove {
                    s.remove(i);
                }

                // ------ Non-cumulative key handling ------
                for field in host_settings.fields().filter(|f| !f.key.is_cumulative()) {
                    // Try to find an existing key in every Directive.
                    //
                    // If found, replace its value non-destructively, otherwise create a new Line.
                    // That way, blank line and comments are preserved.
                    //
                    // When creating a new Line, indent is inferred from the target Section
                    // and line ending is inferred from every Line.
                    match s.get_one_mut(&field.key) {
                        // Line exist -> in-place edit
                        Some(d) => d.set_value(&field.value)?,

                        // Line does not exist, create one and append it to the Section
                        None => {
                            let new_line = Line::directive(field.key.clone(), &field.value)?;
                            s.push(new_line)?;
                        }
                    }
                }

                // Remove lines from the target Section that are not in host_settings.
                // Preserve comments and empty lines.
                //
                // Note: non-cumulative duplicates are kept intact by design
                s.retain(|l| match l.kind() {
                    LineKind::Directive(d) => host_settings.contains_key(&d.field_key()),
                    _ => true,
                });
            }

            // Whole new section
            None => {
                let header = Selector::new(FieldKey::Host, host_settings.host())?
                    .with_ending(inferred_line_ending);

                let mut new_section = Section::new(header).with_ending(inferred_line_ending);
                for field in host_settings.fields() {
                    let param = Line::directive(field.key.clone(), &field.value)?;
                    new_section.push(param)?;
                }
                self.insert_section(0, new_section);
            }
        }
        Ok(())
    }

    /// Return the settings declared under the section whose header value is
    /// exactly `host`.
    ///
    /// No pattern expansion, and no walking of other matching sections. Values
    /// are deduped the way `ssh -G` reads a file: a repeated non-cumulative key
    /// keeps its first occurrence. See [`Self::resolve_host_settings`] for the
    /// full resolution.
    ///
    /// An unknown `host` yields an empty [`HostSettings`], which is
    /// indistinguishable from a section that declares nothing.
    pub fn exact_host_settings(&self, host: &str) -> HostSettings {
        let mut settings = HostSettings::new(host);

        let section = self.section(host);
        if let Some(s) = section {
            for d in s.directives() {
                settings.push_dedup(d.field_key(), d.value());
            }
        }
        settings
    }

    /// Resolve the settings for a given `host`, mimicking `ssh -G` behaviour.
    ///
    /// Unlike [`Self::exact_host_settings`] this expands patterns, walks every
    /// matching section in document order, and applies first-match-wins for
    /// non-cumulative keys.
    ///
    /// # Panics
    /// Not implemented yet, always panics.
    pub fn resolve_host_settings(&self, _host: &str) -> HostSettings {
        todo!("no done yet");
    }
}

impl fmt::Display for SSHConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.preamble {
            write!(f, "{line}")?;
        }
        for section in &self.sections {
            write!(f, "{section}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_host_single_param() {
        let data = "
Host my.server.local
    Hostname 1.2.3.4
";

        let config = SSHConfig::parse(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.field_count(), 1);

        assert!(host_params.contains_key(&FieldKey::Hostname));
        assert_eq!(
            host_params.get_one(&FieldKey::Hostname).unwrap().value,
            "1.2.3.4"
        );
    }

    #[test]
    fn single_host_multiple_params() {
        let data = "
Host my.server.local
    Hostname 1.2.3.4
    User test
";

        let config = SSHConfig::parse(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.field_count(), 2);
        assert!(host_params.contains_key(&FieldKey::Hostname));
        assert_eq!(
            host_params.get_one(&FieldKey::Hostname).unwrap().value,
            "1.2.3.4"
        );
        assert!(host_params.contains_key(&FieldKey::User));
        assert_eq!(host_params.get_one(&FieldKey::User).unwrap().value, "test");
    }

    #[test]
    fn keep_first_duplicated_params() {
        let data = "
Host my.server.local
    User first
    User second
";

        let config = SSHConfig::parse(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.field_count(), 1);
        assert!(host_params.contains_key(&FieldKey::User));
        assert_eq!(host_params.get_one(&FieldKey::User).unwrap().value, "first");
    }

    #[test]
    fn keep_both_cumulative_params() {
        let data = "
Host my.server.local
    IdentityFile ~/.ssh/fake_key1
    IdentityFile ~/.ssh/fake_key2
";

        let config = SSHConfig::parse(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        let cumulative_params: Vec<&Field> = host_params.get_all(&FieldKey::IdentityFile).collect();
        assert_eq!(cumulative_params.len(), 2);
        assert_eq!(cumulative_params[0].value, "~/.ssh/fake_key1");
        assert_eq!(cumulative_params[1].value, "~/.ssh/fake_key2");
    }

    #[test]
    fn match_options_do_not_leak_into_host() {
        let data = "Host a\n\tUser x\nMatch user foo\n\tPort 22\n";
        let config = SSHConfig::parse(data).unwrap();
        let settings = config.exact_host_settings("a");
        assert_eq!(settings.field_count(), 1);
        assert!(!settings.contains_key(&FieldKey::Port));
    }

    #[test]
    fn set_host_settings_creates_missing_host() {
        let mut config = SSHConfig::parse("Host b\n\tUser bob\n").unwrap();
        let mut new_host = HostSettings::new("a");
        new_host.push_dedup(FieldKey::Hostname, "1.2.3.4");
        config.set_host_settings(&new_host).unwrap();

        let a = config.exact_host_settings("a");
        assert_eq!(a.get_one(&FieldKey::Hostname).unwrap().value, "1.2.3.4");
        let b = config.exact_host_settings("b"); // existing host untouched
        assert_eq!(b.get_one(&FieldKey::User).unwrap().value, "bob");
    }

    #[test]
    fn set_host_settings_on_empty_config() {
        let mut config = SSHConfig::parse("").unwrap();
        let mut new_host = HostSettings::new("a");
        new_host.push_dedup(FieldKey::User, "me");
        config.set_host_settings(&new_host).unwrap();
        assert_eq!(
            config
                .exact_host_settings("a")
                .get_one(&FieldKey::User)
                .unwrap()
                .value,
            "me"
        );
    }

    #[test]
    fn full_roundtrip() {
        let lf = "\n";
        let crlf = "\r\n";
        let tab = "\t";
        let spaces = "    ";
        let sep = " ";
        let sep_eq = "=";
        let sep_eq_ws = " = ";
        let trailing_ws = "   ";

        let data = format!(
            "# defaults{lf}\
            AddKeysToAgent{sep}yes{lf}\
            {lf}\
            Host{sep}a{lf}\
            {spaces}HostName{sep}1.2.3.4{lf}\
            {tab}User{sep}test{lf}\
            {lf}\
            host{sep}b{lf}\
            {spaces}HostName{sep}1.2.3.4{lf}\
            {tab}User{sep}test{lf}\
            {lf}\
            Host{sep_eq_ws}b{crlf}\
            {tab}Port{sep_eq}22{crlf}\
            {lf}\
            Host{sep}dev prod *.local{lf}\
            {tab}MyCustomOption{sep}value{lf}\
            {lf}\
            Match{sep}user foo{lf}\
            {tab}Port{sep}22{lf}\
            {lf}\
            Host{sep}*{lf}\
            {tab}IdentityFile{sep}~/.ssh/id{lf}\
            {lf}\
            {trailing_ws}"
        );
        assert_eq!(SSHConfig::parse(&data).unwrap().to_string(), data);
    }

    // --- Line endings ---
    #[test]
    fn infer_line_ending_lf() {
        let config = SSHConfig::parse("Host a\n\tUser x\n").unwrap();
        assert_eq!(config.infer_line_ending().as_str(), "\n");
    }

    #[test]
    fn infer_line_ending_crlf() {
        let config = SSHConfig::parse("Host a\r\n\tUser x\r\n").unwrap();
        assert_eq!(config.infer_line_ending().as_str(), "\r\n");
    }

    #[test]
    fn infer_line_ending_from_comment_only_preamble() {
        // guards the Line::ending widening: no directives anywhere, ending
        // must still be found on a Comment line
        let config = SSHConfig::parse("# managed by hand\r\n").unwrap();
        assert_eq!(config.infer_line_ending().as_str(), "\r\n");
    }

    #[test]
    fn infer_line_ending_uses_document_order() {
        // preamble says LF, section says CRLF: first ending in the file wins
        let config = SSHConfig::parse("AddKeysToAgent yes\nHost a\r\n\tUser x\r\n").unwrap();
        assert_eq!(config.infer_line_ending().as_str(), "\n");
    }

    #[test]
    fn infer_line_ending_defaults_on_empty_config() {
        let config = SSHConfig::parse("").unwrap();
        assert_eq!(config.infer_line_ending(), LineEnding::default());
    }

    #[test]
    fn infer_line_ending_defaults_when_file_has_no_ending() {
        let config = SSHConfig::parse("Host a").unwrap(); // single unterminated line
        assert_eq!(config.infer_line_ending(), LineEnding::default());
    }

    // --- host settings update ---

    #[test]
    fn set_updates_value_in_place_preserving_formatting() {
        let mut config = SSHConfig::parse("Host a\n\tPort=22\n# trailing comment\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::Port, "2222");
        config.set_host_settings(&settings).unwrap();

        // '=' separator, tab indent, and the comment all survive, only the value changed
        assert_eq!(
            config.to_string(),
            "Host a\n\tPort=2222\n# trailing comment\n"
        );
    }

    #[test]
    fn set_matches_existing_key_case_insensitively() {
        let mut config = SSHConfig::parse("Host a\n\thostname 1.1.1.1\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::Hostname, "2.2.2.2");
        config.set_host_settings(&settings).unwrap();

        // lowercase spelling in the file is preserved, no canonicalization on edit
        assert_eq!(config.to_string(), "Host a\n\thostname 2.2.2.2\n");
    }

    #[test]
    fn set_append_key_matching_section_style() {
        let mut config = SSHConfig::parse("Host a\r\n    User x\r\n").unwrap();
        let mut settings = config.exact_host_settings("a");
        settings.push_dedup(FieldKey::Hostname, "1.2.3.4");
        config.set_host_settings(&settings).unwrap();

        // new line copies the section's 4-space indent and the file's CRLF
        assert_eq!(
            config.to_string(),
            "Host a\r\n    User x\r\n    Hostname 1.2.3.4\r\n"
        );
    }

    #[test]
    fn set_creates_missing_host_with_inferred_crlf() {
        let mut config = SSHConfig::parse("Host b\r\n\tUser bob\r\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::Hostname, "1.2.3.4");
        config.set_host_settings(&settings).unwrap();

        // inserted before existing sections, CRLF inferred, existing section untouched
        assert_eq!(
            config.to_string(),
            "Host a\r\n\tHostname 1.2.3.4\r\nHost b\r\n\tUser bob\r\n"
        );
    }

    #[test]
    fn set_creates_host_on_empty_config_with_defaults() {
        let mut config = SSHConfig::parse("").unwrap();
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::User, "me");
        config.set_host_settings(&settings).unwrap();

        let ending = LineEnding::default();
        assert_eq!(
            config.to_string(),
            format!("Host a{ending}\tUser me{ending}")
        );
    }

    #[test]
    fn set_terminates_unterminated_preamble_before_inserting() {
        let mut config = SSHConfig::parse("AddKeysToAgent yes").unwrap();
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::User, "me");
        config.set_host_settings(&settings).unwrap();

        let ending = LineEnding::default();
        assert_eq!(
            config.to_string(),
            format!("AddKeysToAgent yes{ending}Host a{ending}\tUser me{ending}")
        );
    }

    #[test]
    fn set_removes_keys_absent_from_settings() {
        let mut config = SSHConfig::parse(
            "Host a\n\t# keep me\n\tHostname 1.2.3.4\n\tForwardAgent yes\n\tUser x\nHost b\n\tForwardAgent yes\n",
        )
        .unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::Hostname, "1.2.3.4");
        settings.push_dedup(FieldKey::User, "x");
        config.set_host_settings(&settings).unwrap();

        // ForwardAgent removed from host 'a' only, comment and untouched keys keep their bytes
        assert_eq!(
            config.to_string(),
            "Host a\n\t# keep me\n\tHostname 1.2.3.4\n\tUser x\nHost b\n\tForwardAgent yes\n"
        );
    }

    #[test]
    fn cumulative_valid_preserved_divergence_appended() {
        let mut config =
            SSHConfig::parse("Host a\n\tIdentityFile ~/.ssh/key1\n\tIdentityFile ~/.ssh/key2\n")
                .unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::IdentityFile, "~/.ssh/key1");
        settings.push_dedup(FieldKey::IdentityFile, "~/.ssh/keyX");
        config.set_host_settings(&settings).unwrap();

        // key1 line kept in place
        // key2 removed
        // keyX appended
        assert_eq!(
            config.to_string(),
            "Host a\n\tIdentityFile ~/.ssh/key1\n\tIdentityFile ~/.ssh/keyX\n"
        );
    }

    #[test]
    fn cumulative_divergence_preserves_desired_order() {
        let mut config =
            SSHConfig::parse("Host a\n\tIdentityFile A\n\tIdentityFile B\n\tIdentityFile C\n")
                .unwrap();

        let mut settings = HostSettings::new("a");
        for v in ["A", "X", "C"] {
            settings.push_dedup(FieldKey::IdentityFile, v);
        }
        config.set_host_settings(&settings).unwrap();

        // valid_count=1
        // A kept
        // B and C dropped
        // X and C appended in order
        assert_eq!(
            config.to_string(),
            "Host a\n\tIdentityFile A\n\tIdentityFile X\n\tIdentityFile C\n"
        );
    }

    #[test]
    fn cumulative_shrink_removes_extra() {
        let mut config = SSHConfig::parse("Host a\n\tIdentityFile A\n\tIdentityFile B\n").unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::IdentityFile, "A");
        config.set_host_settings(&settings).unwrap();

        assert_eq!(config.to_string(), "Host a\n\tIdentityFile A\n");
    }

    #[test]
    fn cumulative_grow_appends_new() {
        let mut config = SSHConfig::parse("Host a\n\tIdentityFile A\n").unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::IdentityFile, "A");
        settings.push_dedup(FieldKey::IdentityFile, "B");
        config.set_host_settings(&settings).unwrap();

        assert_eq!(
            config.to_string(),
            "Host a\n\tIdentityFile A\n\tIdentityFile B\n"
        );
    }

    #[test]
    fn cumulative_full_match_noop() {
        let data = "Host a\n    IdentityFile A\n    # note\n    IdentityFile B\n";
        let mut config = SSHConfig::parse(data).unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::IdentityFile, "A");
        settings.push_dedup(FieldKey::IdentityFile, "B");
        config.set_host_settings(&settings).unwrap();

        assert_eq!(config.to_string(), data);
    }

    #[test]
    fn cumulative_key_absent_from_desired_is_removed() {
        let mut config =
            SSHConfig::parse("Host a\n\tIdentityFile A\n\tIdentityFile B\n\tUser bob\n").unwrap();

        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::User, "bob");
        config.set_host_settings(&settings).unwrap();

        assert_eq!(config.to_string(), "Host a\n\tUser bob\n");
    }

    #[test]
    fn cumulative_interleaved_keys_both_diverge() {
        let mut config = SSHConfig::parse(
            "Host a\n\tSetEnv A=1\n\tIdentityFile k1\n\tSetEnv B=2\n\tIdentityFile k2\n",
        )
        .unwrap();

        // IdentityFile added first -> processed first -> to_remove pushed as [3, 2] (non-ascending)
        let mut settings = HostSettings::new("a");
        settings.push_dedup(FieldKey::IdentityFile, "k1");
        settings.push_dedup(FieldKey::IdentityFile, "kX");
        settings.push_dedup(FieldKey::SetEnv, "A=1");
        settings.push_dedup(FieldKey::SetEnv, "B=9");
        config.set_host_settings(&settings).unwrap();

        assert_eq!(
            config.to_string(),
            "Host a\n\tSetEnv A=1\n\tIdentityFile k1\n\tIdentityFile kX\n\tSetEnv B=9\n"
        );
    }
}
