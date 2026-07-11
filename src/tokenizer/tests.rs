use super::types::{Token, TokenKind};
use super::parser::tokenize_str;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_single_comment() {
        let data = "# this is a comment";
        let tokens = tokenize_str(data).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Comment);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_whitespace() {
        let data = "      ";
        let tokens = tokenize_str(data).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::WhiteSpace);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_lf() {
        let data = "\n";
        let tokens = tokenize_str(data).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_crlf() {
        let data = "\r\n";
        let tokens = tokenize_str(data).unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, data);
    }

    #[test]
    fn parse_line_ending_crlflf() {
        let data = "\r\n\n";
        let tokens = tokenize_str(data).unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].kind, TokenKind::LineEnding);
        assert_eq!(tokens[0].data, "\r\n");
        assert_eq!(tokens[1].kind, TokenKind::LineEnding);
        assert_eq!(tokens[1].data, "\n");
    }

    #[test]
    fn err_on_line_ending_cr() {
        let data = "\r";
        let err = tokenize_str(data);
        assert!(err.is_err());
    }

    #[test]
    fn parse_key_single_whitespace_value() {
        let key = "Host";
        let sep = " ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_multiple_whitespace_value() {
        let key = "Host";
        let sep = "   ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_equal_value() {
        let key = "Host";
        let sep = "=";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

        #[test]
    fn parse_key_sep_quoted_value() {
        let key = "Host";
        let sep = "=";
        let value = "\"my.host\"";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn err_on_unclosed_double_quote() {
        let key = "Host";
        let sep = "=";
        let value = "\"test";
        let data = format!("{key}{sep}{value}");

        let err = tokenize_str(&data);
        assert!(err.is_err());
    }

    #[test]
    fn parse_key_equal_whitespace_value() {
        let key = "Host";
        let sep = "=  ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_whitespace_equal_value() {
        let key = "Host";
        let sep = "  =";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn parse_key_equal_whitespace_equal_value() {
        let key = "Host";
        let sep = "  =  ";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let tokens = tokenize_str(&data).unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Key);
        assert_eq!(tokens[0].data, key);
        assert_eq!(tokens[1].kind, TokenKind::Separator);
        assert_eq!(tokens[1].data, sep);
        assert_eq!(tokens[2].kind, TokenKind::Value);
        assert_eq!(tokens[2].data, value);
    }

    #[test]
    fn err_on_key_two_equal_value() {
        let key = "Host";
        let sep = "==";
        let value = "my.host";
        let data = format!("{key}{sep}{value}");

        let err = tokenize_str(&data);
        assert!(err.is_err());
    }
}