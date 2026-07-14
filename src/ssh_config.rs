use crate::field_keys::FieldKey;
use crate::lexer::Lexer;
use crate::line::{Directive, Line};
use crate::section::Section;
use crate::settings::{Field, HostSettings};
use std::fmt;

const DEFAULT_LINE_ENDING: &str = "\n";
const DEFAULT_LINE_INDENT: &str = "\t";

pub struct SSHConfig {
    preamble: Vec<Line>,
    sections: Vec<Section>,
}

impl SSHConfig {
    pub fn new(data: &str) -> Result<SSHConfig, String> {
        let lexer = Lexer::new(&data);
        let lines = Line::parse_lines(lexer.tokenize()?)?;
        let (preamble, sections) = Section::parse_sections(lines);
        return Ok(SSHConfig { preamble, sections });
    }

    /// Infer line ending from the preamble and every section header.
    ///
    /// Default to '\n' if no line ending is found.
    fn infer_line_ending(&self) -> String {
        self.preamble
            .iter()
            .find_map(Line::ending)
            .or_else(|| self.sections.iter().find_map(Section::ending))
            .map_or_else(|| DEFAULT_LINE_ENDING.to_string(), |t| t.data.clone())
    }

    pub fn set_host_settings(&mut self, host_settings: &HostSettings) -> Result<(), String> {
        let inferred_line_ending = self.infer_line_ending();
        let target_section = self
            .sections
            .iter_mut()
            .find(|s| s.header.value.data == host_settings.host);

        match target_section {
            Some(s) => {
                let inferred_indent = s
                    .indent()
                    .map_or_else(|| DEFAULT_LINE_INDENT.to_string(), |t| t.data.clone());
                for field in &host_settings.fields {
                    if field.key.is_cumulative() {
                        todo!("no done yet");
                    } else {
                        // Try to find an existing key in every Line::Directive.
                        //
                        // If found, replace its value non-destructively, otherwise create a new Line.
                        // That way, blank line and comments are preserved.
                        //
                        // When creating a new Line, indent is inferred from the target Section
                        // and line ending is inferred from every Line.
                        let existing_line = s.body.iter_mut().find_map(|l| match l {
                            Line::Directive(d) if FieldKey::parse(&d.key.data) == field.key => {
                                Some(d)
                            }
                            _ => None,
                        });

                        match existing_line {
                            // Line exist -> in-place edit
                            Some(l) => l.value.data = field.value.clone(),

                            // Line does not exist, create one and append it to the Section
                            _ => {
                                let new_directive =
                                    Directive::new(field.key.as_canonical_str(), &field.value)?
                                        .with_indent(&inferred_indent)?
                                        .with_ending(&inferred_line_ending)?;
                                let new_line = Line::Directive(new_directive);
                                s.push_line(new_line, &inferred_line_ending)?;
                            }
                        }
                    }
                }
            }
            None => {
                // Add a line ending to the preamble's last line if none where found.
                if let Some(last_line) = self.preamble.last_mut()
                    && last_line.ending().is_none()
                {
                    last_line.set_ending(&inferred_line_ending)?;
                }

                let header = Directive::new(&FieldKey::Host.to_string(), &host_settings.host)?
                    .with_ending(&inferred_line_ending)?;

                let mut new_section = Section {
                    header,
                    body: Vec::new(),
                };
                for field in &host_settings.fields {
                    let param = Directive::new(&field.key.to_string(), &field.value)?
                        .with_indent(DEFAULT_LINE_INDENT)?
                        .with_ending(&inferred_line_ending)?;
                    new_section.push_line(Line::Directive(param), &inferred_line_ending)?;
                }
                self.sections.insert(0, new_section);
            }
        }
        Ok(())
    }

    /// Return the settings declared under the `Host` exactly matching the provided `host`.
    ///
    /// Note: matches only a literal exact `Host` value.
    pub fn exact_host_settings(&self, host: &str) -> HostSettings {
        let directives = self
            .sections
            .iter()
            .find(|s| s.header.value.data == host)
            .into_iter()
            .flat_map(|s| &s.body)
            .filter_map(|l| match l {
                Line::Directive(d) => Some(d),
                _ => None,
            });

        let mut settings = HostSettings::new(host);
        for d in directives {
            settings.add_field(Field {
                key: FieldKey::parse(&d.key.data),
                value: d.value.data.clone(),
            });
        }
        settings
    }

    // Resolve the settings for a given `host` mimicking `ssh -G` behaviour.
    pub fn resolve_host_settings(&self, host: &str) -> HostSettings {
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

        let config = SSHConfig::new(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.len(), 1);

        assert!(host_params.contains_key(&FieldKey::Hostname));
        assert_eq!(host_params.get_one(&FieldKey::Hostname).unwrap(), "1.2.3.4");
    }

    #[test]
    fn single_host_multiple_params() {
        let data = "
Host my.server.local
    Hostname 1.2.3.4
    User test
";

        let config = SSHConfig::new(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.len(), 2);
        assert!(host_params.contains_key(&FieldKey::Hostname));
        assert_eq!(host_params.get_one(&FieldKey::Hostname).unwrap(), "1.2.3.4");
        assert!(host_params.contains_key(&FieldKey::User));
        assert_eq!(host_params.get_one(&FieldKey::User).unwrap(), "test");
    }

    #[test]
    fn keep_first_duplicated_params() {
        let data = "
Host my.server.local
    User first
    User second
";

        let config = SSHConfig::new(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        assert_eq!(host_params.len(), 1);
        assert!(host_params.contains_key(&FieldKey::User));
        assert_eq!(host_params.get_one(&FieldKey::User).unwrap(), "first");
    }

    #[test]
    fn keep_both_cumulative_params() {
        let data = "
Host my.server.local
    IdentityFile ~/.ssh/fake_key1
    IdentityFile ~/.ssh/fake_key2
";

        let config = SSHConfig::new(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        let cumulative_params = host_params.get_multiple(&FieldKey::IdentityFile);
        assert_eq!(cumulative_params.len(), 2);
        assert_eq!(cumulative_params[0], "~/.ssh/fake_key1");
        assert_eq!(cumulative_params[1], "~/.ssh/fake_key2");
    }

    #[test]
    fn match_options_do_not_leak_into_host() {
        let data = "Host a\n\tUser x\nMatch user foo\n\tPort 22\n";
        let config = SSHConfig::new(data).unwrap();
        let settings = config.exact_host_settings("a");
        assert_eq!(settings.len(), 1);
        assert!(!settings.contains_key(&FieldKey::Port));
    }

    #[test]
    fn set_host_settings_creates_missing_host() {
        let mut config = SSHConfig::new("Host b\n\tUser bob\n").unwrap();
        let mut new_host = HostSettings::new("a");
        new_host.add_field(Field {
            key: FieldKey::Hostname,
            value: "1.2.3.4".into(),
        });
        config.set_host_settings(&new_host).unwrap();

        let a = config.exact_host_settings("a");
        assert_eq!(a.get_one(&FieldKey::Hostname), Some("1.2.3.4"));
        let b = config.exact_host_settings("b"); // existing host untouched
        assert_eq!(b.get_one(&FieldKey::User), Some("bob"));
    }

    #[test]
    fn set_host_settings_on_empty_config() {
        let mut config = SSHConfig::new("").unwrap();
        let mut new_host = HostSettings::new("a");
        new_host.add_field(Field {
            key: FieldKey::User,
            value: "me".into(),
        });
        config.set_host_settings(&new_host).unwrap();
        assert_eq!(
            config.exact_host_settings("a").get_one(&FieldKey::User),
            Some("me")
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
        assert_eq!(SSHConfig::new(&data).unwrap().to_string(), data);
    }

    // --- Line endings ---
    #[test]
    fn infer_line_ending_lf() {
        let config = SSHConfig::new("Host a\n\tUser x\n").unwrap();
        assert_eq!(config.infer_line_ending(), "\n");
    }

    #[test]
    fn infer_line_ending_crlf() {
        let config = SSHConfig::new("Host a\r\n\tUser x\r\n").unwrap();
        assert_eq!(config.infer_line_ending(), "\r\n");
    }

    #[test]
    fn infer_line_ending_from_comment_only_preamble() {
        // guards the Line::ending widening: no directives anywhere, ending
        // must still be found on a Comment line
        let config = SSHConfig::new("# managed by hand\r\n").unwrap();
        assert_eq!(config.infer_line_ending(), "\r\n");
    }

    #[test]
    fn infer_line_ending_uses_document_order() {
        // preamble says LF, section says CRLF: first ending in the file wins
        let config = SSHConfig::new("AddKeysToAgent yes\nHost a\r\n\tUser x\r\n").unwrap();
        assert_eq!(config.infer_line_ending(), "\n");
    }

    #[test]
    fn infer_line_ending_defaults_on_empty_config() {
        let config = SSHConfig::new("").unwrap();
        assert_eq!(config.infer_line_ending(), DEFAULT_LINE_ENDING);
    }

    #[test]
    fn infer_line_ending_defaults_when_file_has_no_ending() {
        let config = SSHConfig::new("Host a").unwrap(); // single unterminated line
        assert_eq!(config.infer_line_ending(), DEFAULT_LINE_ENDING);
    }

    // --- host settings update ---

    #[test]
    fn set_updates_value_in_place_preserving_formatting() {
        let mut config = SSHConfig::new("Host a\n\tPort=22\n# trailing comment\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::Port,
            value: "2222".into(),
        });
        config.set_host_settings(&settings).unwrap();

        // '=' separator, tab indent, and the comment all survive, only the value changed
        assert_eq!(
            config.to_string(),
            "Host a\n\tPort=2222\n# trailing comment\n"
        );
    }

    #[test]
    fn set_matches_existing_key_case_insensitively() {
        let mut config = SSHConfig::new("Host a\n\thostname 1.1.1.1\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::Hostname,
            value: "2.2.2.2".into(),
        });
        config.set_host_settings(&settings).unwrap();

        // lowercase spelling in the file is preserved, no canonicalization on edit
        assert_eq!(config.to_string(), "Host a\n\thostname 2.2.2.2\n");
    }

    #[test]
    fn set_appends_missing_key_matching_section_style() {
        let mut config = SSHConfig::new("Host a\r\n    User x\r\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::Hostname,
            value: "1.2.3.4".into(),
        });
        config.set_host_settings(&settings).unwrap();

        // new line copies the section's 4-space indent and the file's CRLF
        assert_eq!(
            config.to_string(),
            "Host a\r\n    User x\r\n    Hostname 1.2.3.4\r\n"
        );
    }

    #[test]
    fn set_creates_missing_host_with_inferred_crlf() {
        let mut config = SSHConfig::new("Host b\r\n\tUser bob\r\n").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::Hostname,
            value: "1.2.3.4".into(),
        });
        config.set_host_settings(&settings).unwrap();

        // inserted before existing sections, CRLF inferred, existing section untouched
        assert_eq!(
            config.to_string(),
            "Host a\r\n\tHostname 1.2.3.4\r\nHost b\r\n\tUser bob\r\n"
        );
    }

    #[test]
    fn set_creates_host_on_empty_config_with_defaults() {
        let mut config = SSHConfig::new("").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::User,
            value: "me".into(),
        });
        config.set_host_settings(&settings).unwrap();

        assert_eq!(config.to_string(), "Host a\n\tUser me\n");
    }

    #[test]
    fn set_terminates_unterminated_preamble_before_inserting() {
        let mut config = SSHConfig::new("AddKeysToAgent yes").unwrap();
        let mut settings = HostSettings::new("a");
        settings.add_field(Field {
            key: FieldKey::User,
            value: "me".into(),
        });
        config.set_host_settings(&settings).unwrap();

        assert_eq!(
            config.to_string(),
            "AddKeysToAgent yes\nHost a\n\tUser me\n"
        );
    }
}
