use crate::lexer::{Lexer, Token, TokenKind};
use crate::settings::{HostSettings, Field};
use crate::field_keys::FieldKey;


pub struct SshConfig {
    tokens: Vec<Token>
}

impl SshConfig {
    pub fn new(data: &str) -> Result<SshConfig, String>{
        let lexer = Lexer::new(&data);

        return Ok(SshConfig {
            tokens: lexer.tokenize()?
        });
    }

    /// Return the settings declared under the `Host` exactly matching the provided `host`.
    /// 
    /// Note: matches only a literal exact `Host` value.
    pub fn exact_host_settings(&self, host: &str) -> HostSettings {
        let mut host_settings= HostSettings::new(host);
        let mut in_target_section = false;

        let ksv_tokens: Vec<&Token> = self.tokens.iter()
            .filter(|t| matches!(
                t.kind,
                TokenKind::FieldKey | TokenKind::FieldSeparator | TokenKind::FieldValue
            ))
            .collect();

        for chunk in ksv_tokens.chunks_exact(3) {
            let [key, sep, val] = chunk else { continue; };
            if FieldKey::parse(&key.data) == FieldKey::Host {
                // break when the literal 'Host' section is done
                if in_target_section {
                    break
                }
                in_target_section = val.data == host;
            } else if in_target_section {
                host_settings.add_field(Field {
                    key: FieldKey::parse(&key.data),
                    separator: sep.data.clone(),
                    value: val.data.clone()
                });
            }
        }
        return host_settings;
    }

    /// Resolve the settings for a given `host` mimicking `ssh -G` behaviour.
    pub fn resolve_host_settings(&self, host: &str) -> HostSettings {
        let mut host_settings= HostSettings::new(host);
        let mut in_target_section = false;

        let ksv_tokens: Vec<&Token> = self.tokens.iter()
            .filter(|t| matches!(
                t.kind,
                TokenKind::FieldKey | TokenKind::FieldSeparator | TokenKind::FieldValue
            ))
            .collect();

        for chunk in ksv_tokens.chunks_exact(3) {
            let [key, sep, val] = chunk else { continue; };
            if FieldKey::parse(&key.data) == FieldKey::Host {
                in_target_section = val.data == host;
            } else if in_target_section {
                host_settings.add_field(Field {
                    key: FieldKey::parse(&key.data),
                    separator: sep.data.clone(),
                    value: val.data.clone()
                });
            }
        }
        return host_settings;
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