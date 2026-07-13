use crate::field_keys::FieldKey;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::line::{Directive, Line};
use crate::section::Section;
use crate::settings::{Field, HostSettings};

pub struct SshConfig {
    preamble: Vec<Line>,
    sections: Vec<Section>,
}

impl SshConfig {
    pub fn new(data: &str) -> Result<SshConfig, String> {
        let lexer = Lexer::new(&data);
        let lines = Line::parse_lines(lexer.tokenize()?)?;
        let (preamble, sections) = Section::parse_sections(lines);
        return Ok(SshConfig { preamble, sections });
    }

    pub fn set_host_settings(&mut self, host_settings: &HostSettings) -> Result<(), String> {
        let target_section = self
            .sections
            .iter_mut()
            .find(|s| s.header.value.data == host_settings.host);

        match target_section {
            Some(s) => {}
            None => {
                let header = Directive::new(&FieldKey::Host.to_string(), &host_settings.host)?
                    .with_ending("\n")?;

                let mut body: Vec<Line> = Vec::new();
                for field in &host_settings.fields {
                    let param = Directive::new(&field.key.to_string(), &field.value)?
                        .with_indent("\t")?
                        .with_ending("\n")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_host_single_param() {
        let data = "
Host my.server.local
    Hostname 1.2.3.4
";

        let config = SshConfig::new(data).unwrap();
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

        let config = SshConfig::new(data).unwrap();
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

        let config = SshConfig::new(data).unwrap();
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

        let config = SshConfig::new(data).unwrap();
        let host_params = config.exact_host_settings("my.server.local");
        let cumulative_params = host_params.get_multiple(&FieldKey::IdentityFile);
        assert_eq!(cumulative_params.len(), 2);
        assert_eq!(cumulative_params[0], "~/.ssh/fake_key1");
        assert_eq!(cumulative_params[1], "~/.ssh/fake_key2");
    }
}
