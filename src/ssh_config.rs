use crate::field_keys::FieldKey;
use crate::lexer::{Lexer, Token, TokenKind};
use crate::settings::{Field, HostSettings};
use crate::line::Line;
use crate::section::Section;

pub struct SshConfig {
    preamble: Vec<Line>,
    sections: Vec<Section>
}

impl SshConfig {
    pub fn new(data: &str) -> Result<SshConfig, String> {
        let lexer = Lexer::new(&data);
        let lines = Line::parse_lines(lexer.tokenize()?)?;
        let (preamble, sections) = Section::parse_sections(lines);
        return Ok(SshConfig {
            preamble,
            sections
        });
    }

    pub fn set_host_settings(&mut self, host_settings: &HostSettings) {
        // Infer line ending to use when saving
        // let line_ending_token = self
        //     .tokens
        //     .iter()
        //     .find(|t| matches!(t.kind, TokenKind::LineEnding))
        //     .map_or(
        //         Token {
        //             kind: TokenKind::LineEnding,
        //             data: '\n'.into(),
        //         },
        //         |t| t.clone(),
        //     );

        // // Build the token representation HostSettings
        // let mut new_host_section: Vec<Token> = Vec::new();
        // new_host_section.push(Token {
        //     kind: TokenKind::FieldKey,
        //     data: FieldKey::Host.to_string(),
        // });
        // new_host_section.push(Token {
        //     kind: TokenKind::FieldSeparator,
        //     data: " ".into(),
        // });
        // new_host_section.push(Token {
        //     kind: TokenKind::FieldValue,
        //     data: host_settings.host.clone(),
        // });
        // new_host_section.push(line_ending_token.clone());

        // for field in &host_settings.fields {
        //     new_host_section.push(Token {
        //         kind: TokenKind::WhiteSpace,
        //         data: '\t'.into(),
        //     });
        //     new_host_section.push(Token {
        //         kind: TokenKind::FieldKey,
        //         data: field.key.to_string(),
        //     });
        //     new_host_section.push(Token {
        //         kind: TokenKind::FieldSeparator,
        //         data: field.separator.clone(),
        //     });
        //     new_host_section.push(Token {
        //         kind: TokenKind::FieldValue,
        //         data: field.value.clone(),
        //     });
        //     new_host_section.push(line_ending_token.clone());
        // }

        // let ksv_tokens: Vec<(usize, &Token)> = self
        //     .tokens
        //     .iter()
        //     .enumerate()
        //     .filter(|(_, t)| {
        //         matches!(
        //             t.kind,
        //             TokenKind::FieldKey | TokenKind::FieldSeparator | TokenKind::FieldValue
        //         )
        //     })
        //     .collect();

        // // Find the start and end index of the target host section.
        // // Insert at the top if none is found
        // let mut host_section_found = false;
        // let mut host_section_start: usize = 0;
        // let mut host_section_end: usize = 0;
        // for chunk in ksv_tokens.chunks_exact(3) {
        //     let [(index, key), _, (_, value)] = chunk else {
        //         continue;
        //     };
        //     if FieldKey::parse(&key.data) == FieldKey::Host {
        //         if host_section_found {
        //             host_section_end = *index;
        //             break;
        //         }

        //         if value.data == host_settings.host {
        //             host_section_start = *index;
        //             host_section_found = true;
        //         } else {
        //             host_section_found = false;
        //         }
        //     }
        // }

        // self.tokens
        //     .splice(host_section_start..host_section_end, new_host_section);
    }

    /// Return the settings declared under the `Host` exactly matching the provided `host`.
    ///
    /// Note: matches only a literal exact `Host` value.
    pub fn exact_host_settings(&self, host: &str) -> HostSettings {
        let directives = self.sections
            .iter()
            .find(|s| s.header.value.data == host)
            .into_iter()
            .flat_map(|s| &s.body)
            .filter_map(|l| match l {
                Line::Directive(d) => Some(d),
                _ => None
            });

        let mut settings = HostSettings::new(host);
        for d in directives {
            settings.add_field(Field { 
                key: FieldKey::parse(&d.key.data), 
                value: d.value.data.clone() 
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
