use crate::lexer::{Lexer, Token, TokenKind};
use crate::field::HostFields;


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

    /// Query fields for a given host while mimicking `ssh -G` behaviour
    pub fn query_host_fields(&self, host: &str) -> HostFields {
        let mut host_fields= HostFields::new();
        let mut in_target_section = false;

        let ksv_tokens: Vec<&Token> = self.tokens.iter()
            .filter(|t| matches!(
                t.kind,
                TokenKind::FieldKey | TokenKind::FieldSeparator | TokenKind::FieldValue
            ))
            .collect();

        for chunk in ksv_tokens.chunks_exact(3) {
            let [key, _, val] = chunk else { continue; };
            if key.data.eq_ignore_ascii_case("Host") {
                in_target_section = val.data == host;
            } else if in_target_section {
                host_fields.add_field(&key.data, &val.data);
            }
        }
        return host_fields;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_single_host_single_param() {
        let data = "
Host my.server.local
    Key1 Value1
";

        let config = SshConfig::new(data).unwrap();
        let host_params = config.query_host_fields("my.server.local");
        assert_eq!(host_params.len(), 1);
        assert!(host_params.contains_key("Key1"));
        assert_eq!(host_params.get_one("Key1").unwrap(), "Value1");
    }

    #[test]
    fn query_single_host_multiple_params() {
        let data = "
Host my.server.local
    Key1 Value1
    Key2 Value2
";

        let config = SshConfig::new(data).unwrap();
        let host_params = config.query_host_fields("my.server.local");
        assert_eq!(host_params.len(), 2);
        assert!(host_params.contains_key("Key1"));
        assert_eq!(host_params.get_one("Key1").unwrap(), "Value1");
        assert!(host_params.contains_key("Key2"));
        assert_eq!(host_params.get_one("Key2").unwrap(), "Value2");
    }

    #[test]
    fn keep_first_duplicated_params() {
        let data = "
Host my.server.local
    Key1 Value1
    Key1 Value2
";

        let config = SshConfig::new(data).unwrap();
        let host_params = config.query_host_fields("my.server.local");
        assert_eq!(host_params.len(), 1);
        assert_eq!(host_params.get_one("Key1").unwrap(), "Value1");
    }
}