use crate::field_keys::FieldKey;
use crate::lexer::Lexer;
use crate::line::{Directive, Line};
use crate::section::Section;
use crate::settings::{Field, HostSettings};
use std::fmt;

const DEFAULT_LINE_ENDING: &str = "\n";

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
        self
            .preamble
            .iter()
            .find_map(Line::ending)
            .or_else(|| self.sections.iter().find_map(Section::ending))
            .map_or_else(|| DEFAULT_LINE_ENDING.to_string(), |t| t.data.clone())
    }

    pub fn set_host_settings(&mut self, host_settings: &HostSettings) -> Result<(), String> {
        let target_section = self
            .sections
            .iter_mut()
            .find(|s| s.header.value.data == host_settings.host);

        match target_section {
            Some(s) => {
                // TODO handle partial edits
            }
            None => {
                let line_ending = self.infer_line_ending();
                let header = Directive::new(&FieldKey::Host.to_string(), &host_settings.host)?
                    .with_ending(&line_ending)?;

                let mut body: Vec<Line> = Vec::new();
                for field in &host_settings.fields {
                    let param = Directive::new(&field.key.to_string(), &field.value)?
                        .with_indent("\t")?
                        .with_ending(&line_ending)?;
                    body.push(Line::Directive(param));
                }
                self.sections.insert(0, Section { header, body });
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
    // pub fn resolve_host_settings(&self, host: &str) -> HostSettings {
    //     // TODO
    // }
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
}
