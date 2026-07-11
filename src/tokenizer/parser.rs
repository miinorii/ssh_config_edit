use std::{error::Error, fs};
use super::types::{Token, TokenKind};

pub fn tokenize_file(path: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let data= fs::read_to_string(path)?;
    let result = tokenize_str(&data)?;
    return Ok(result);
}

/// https://man7.org/linux/man-pages/man5/ssh_config.5.html
pub fn tokenize_str(data: &str) -> Result<Vec<Token>, String> {
    let mut iter = data.char_indices().peekable();

    let mut parsed_tokens: Vec<Token> = Vec::new();
    let mut current_line = 1; // 1-indexed line number
    let mut current_pos = 1; // 1-indexed position in the current line, used for logging and errors
    while let Some((starting_pos, starting_char)) = iter.next() {
        current_pos += 1;
        match starting_char {
            // Comments
            '#' => {
                let mut byte_offset = starting_char.len_utf8();
                while let Some(&(_, char)) = iter.peek() {
                    // consider a comment end when on a newline or when there is no more data
                    if char == '\n' || char == '\r' {
                        break;
                    }
                    iter.next();
                    current_pos += 1;
                    byte_offset += char.len_utf8();
                }
                parsed_tokens.push(Token {
                    kind: TokenKind::Comment,
                    data: data[starting_pos..starting_pos+byte_offset].to_string()
                });
                
            },

            // Line endings
            '\n' => {
                parsed_tokens.push(Token { kind: TokenKind::LineEnding, data: data[starting_pos..starting_pos+1].to_string()});
                current_line += 1;
            },
            '\r' => {
                let next = iter.peek();
                match next {
                    Some((_, '\n')) => {
                        iter.next();
                        current_pos += 1;
                        current_line += 1;
                        parsed_tokens.push(Token { kind: TokenKind::LineEnding, data: data[starting_pos..starting_pos+2].to_string()});
                    },
                    Some((_, other)) => {
                        return Err(format!("at ln:{} pos:{}, expected '\n' found '{}'", current_line, current_pos+1, other).into());
                    },
                    None => {
                        return Err(format!("at ln:{} pos:{}, file ended too soon", current_line, current_pos+1).into());
                    }
                }
            }

            // First level Whitespaces
            whitespace_char if whitespace_char.is_whitespace() => {
                let mut byte_offset = whitespace_char.len_utf8();
                while let Some(&(_, char)) = iter.peek() {
                    if !char.is_whitespace() {
                        break;
                    }
                    iter.next();
                    current_pos += 1;
                    byte_offset += char.len_utf8();
                }
                parsed_tokens.push(Token { 
                    kind: TokenKind::WhiteSpace, 
                    data: data[starting_pos..starting_pos+byte_offset].to_string()
                })
            },

            // Other: key + separator + value
            other_char => {
                // extract the key
                let mut key_end_byte_offset = other_char.len_utf8();
                loop {
                    let next_char = iter.peek();
                    match next_char {
                        // end if key boundary can't be detected (improper file format)
                        Some((_, '\r')) | Some((_, '\n')) | None => {
                            return Err(format!("at ln:{} pos:{}, key boundary not found", current_line, current_pos+1));
                        },
                        // key boundary
                        Some((_, c)) if c.is_whitespace() || *c == '=' => {
                            break;
                        },
                        // key content
                        Some((_, c)) => {
                            key_end_byte_offset += c.len_utf8();
                            current_pos += 1;
                            iter.next();
                        },
                    }
                }
                parsed_tokens.push(Token {
                    kind: TokenKind::Key,
                    data: data[starting_pos..starting_pos+key_end_byte_offset].to_string()
                });

                // extract the separator
                let mut separator_end_byte_offset = key_end_byte_offset;
                let mut equal_seen = false;
                loop {
                    let next_char = iter.peek();
                    match next_char {
                        // end if separator boundary can't be detected (improper file format)
                        Some((_, '\r')) | Some((_, '\n')) | None => {
                            return Err(format!("at ln:{} pos:{}, separator boundary not found", current_line, current_pos+1));
                        },

                        // first time we see '='
                        Some((_, c @ '=')) if !equal_seen => {
                            equal_seen = true;
                            separator_end_byte_offset += c.len_utf8();
                            current_pos += 1;
                            iter.next();
                        },

                        // if we see '=' another time
                        Some((_, '=')) => {
                            return Err(format!("at ln:{} pos:{}, separator contain two '='", current_line, current_pos+1));
                        },

                        // separator boundary
                        Some((_, c)) if !c.is_whitespace() => {
                            break;
                        },

                        // separator content
                        Some((_, c)) => {
                            separator_end_byte_offset += c.len_utf8();
                            current_pos += 1;
                            iter.next();
                        },
                    }
                }

                let separator_start = starting_pos+key_end_byte_offset;
                let separator_end = starting_pos+separator_end_byte_offset;
                parsed_tokens.push(Token {
                    kind: TokenKind::Separator,
                    data: data[separator_start..separator_end].to_string()
                });
                
                // extract the value
                let mut value_end_byte_offset = separator_end_byte_offset;
                loop {
                    let next_char = iter.peek();
                    match next_char {
                        // end if value boundary can't be detected (improper file format)
                        Some((_, '\r')) | Some((_, '\n')) | None if value_end_byte_offset == separator_end_byte_offset => {
                            return Err(format!("at ln:{} pos:{}, value not provided", current_line, current_pos+1));
                        },

                        // value boundary
                        Some((_, '\r')) | Some((_, '\n')) | None => {
                            break;
                        },

                        // key content
                        Some((_, c)) => {
                            value_end_byte_offset += c.len_utf8();
                            current_pos += 1;
                            iter.next();
                        },


                    }
                }

                let value_start = starting_pos+separator_end_byte_offset;
                let value_end = starting_pos+value_end_byte_offset;
                let value_data = data[value_start..value_end].to_string();

                // detect unclosed double quotes naively by checking if there is an odd count
                if value_data.chars().filter(|c| *c == '"').count() % 2 != 0 {
                    return Err(format!("at ln:{} pos:{}, unclosed double quote", current_line, value_start));
                }
                parsed_tokens.push(Token {
                    kind: TokenKind::Value,
                    data: value_data
                });
            }
        }
    }
    
    Ok(parsed_tokens)
}


