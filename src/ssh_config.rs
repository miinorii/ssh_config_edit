use std::error::Error;
use std::fs;
use std::collections::HashMap;
use crate::tokenizer::{lexer::Lexer, types::{Token, TokenKind}};

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

    pub fn query_host(&self, host: &str) -> HashMap<String, String> {
        let mut host_params: HashMap<String, String> = HashMap::new();
        let mut in_target_section = false;

        let ksv_tokens: Vec<&Token> = self.tokens.iter()
            .filter(|t| matches!(
                t.kind,
                TokenKind::FieldKey | TokenKind::FieldSeparator | TokenKind::FieldValue
            ))
            .collect();

        for chunk in ksv_tokens.chunks_exact(3) {
            let [key, _, val] = chunk else { continue; };
            if key.data.to_lowercase() == "host" {
                in_target_section = val.data == host;
            } else if in_target_section && !host_params.contains_key(&key.data.to_lowercase()){
                host_params.insert(key.data.to_lowercase(), val.data.clone());
            }
        }
        return host_params;
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
        let host_params = config.query_host("my.server.local");
        assert_eq!(host_params.len(), 1);
        assert!(host_params.contains_key("key1"));
        assert_eq!(host_params.get("key1").unwrap(), "Value1");
    }

    #[test]
    fn query_single_host_multiple_params() {
        let data = "
Host my.server.local
    Key1 Value1
    Key2 Value2
";

        let config = SshConfig::new(data).unwrap();
        let host_params = config.query_host("my.server.local");
        assert_eq!(host_params.len(), 2);
        assert!(host_params.contains_key("key1"));
        assert_eq!(host_params.get("key1").unwrap(), "Value1");
        assert!(host_params.contains_key("key2"));
        assert_eq!(host_params.get("key2").unwrap(), "Value2");
    }

    #[test]
    fn keep_first_duplicated_params() {
        let data = "
Host my.server.local
    Key1 Value1
    Key1 Value2
";

        let config = SshConfig::new(data).unwrap();
        let host_params = config.query_host("my.server.local");
        assert_eq!(host_params.len(), 1);
        assert_eq!(host_params.get("key1").unwrap(), "Value1");
    }
}