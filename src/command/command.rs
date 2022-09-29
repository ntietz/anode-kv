use thiserror::Error;

use crate::codec::Token;
use crate::types::{Key, Value};

#[derive(Debug, PartialEq)]
pub enum Command {
    Echo(Value),
    Command,
    Get(Key),
    Set(Key, Value),
}

#[derive(Error, Debug, PartialEq)]
pub enum CommandError {
    #[error("insufficient tokens")]
    InsufficientTokens,

    #[error("malformed input")]
    Malformed,

    #[error("unknown command {0}")]
    UnknownCommand(String),
}

impl Command {
    pub fn from_tokens(tokens: &[Token]) -> Result<(Command, usize), CommandError> {
        log::info!("parsing tokens: {:?}", tokens);
        if tokens.len() == 0 {
            return Err(CommandError::InsufficientTokens);
        }

        let length = match tokens[0] {
            Token::Array(l) if l > 0 => l as usize,
            _ => return Err(CommandError::Malformed),
        };
        log::info!("length: {}", length);

        if tokens.len() - 1 < length {
            return Err(CommandError::InsufficientTokens);
        }

        let cmd = match &tokens[1] {
            Token::SimpleString(c) => c.to_uppercase(),
            Token::BulkString(Some(c)) => match std::str::from_utf8(c) {
                Ok(s) => s.to_uppercase(),
                Err(_) => return Err(CommandError::Malformed),
            },
            _ => return Err(CommandError::Malformed),
        };
        log::info!("length: {}", length);

        match cmd.as_str() {
            "ECHO" => {
                if length != 2 {
                    return Err(CommandError::Malformed);
                }
                let reply_token = match &tokens[2] {
                    Token::SimpleString(s) => s.bytes().collect(),
                    Token::BulkString(Some(s)) => s.clone(),
                    _ => return Err(CommandError::Malformed),
                };
                Ok((Command::Echo(reply_token.into()), 3))
            }
            "COMMAND" => {
                if length != 1 {
                    return Err(CommandError::Malformed);
                }
                Ok((Command::Command, 2))
            }
            "GET" => {
                if length != 2 {
                    return Err(CommandError::Malformed);
                }
                let key = match &tokens[2] {
                    Token::SimpleString(s) => s.bytes().collect(),
                    Token::BulkString(Some(s)) => s.clone(),
                    _ => return Err(CommandError::Malformed),
                };

                Ok((Command::Get(key.into()), 3))
            }
            "SET" => {
                if length != 3 {
                    return Err(CommandError::Malformed);
                }
                let key = match &tokens[2] {
                    Token::SimpleString(s) => s.bytes().collect(),
                    Token::BulkString(Some(s)) => s.clone(),
                    _ => return Err(CommandError::Malformed),
                };
                let value = match &tokens[3] {
                    Token::SimpleString(s) => s.bytes().collect(),
                    Token::BulkString(Some(s)) => s.clone(),
                    _ => return Err(CommandError::Malformed),
                };

                Ok((Command::Set(key.into(), value.into()), 4))
            }
            unk => Err(CommandError::UnknownCommand(unk.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_errors_for_empty_input() {
        let input = vec![];
        let expected = Err(CommandError::InsufficientTokens);

        assert_eq!(expected, Command::from_tokens(&input));
    }

    #[test]
    fn it_has_errors_for_incomplete_input() {
        let input = vec![Token::Array(2), Token::SimpleString("echo".to_string())];
        let expected = Err(CommandError::InsufficientTokens);

        assert_eq!(expected, Command::from_tokens(&input));
    }

    #[test]
    fn it_has_errors_for_malformed_input() {
        let input = vec![Token::SimpleString("whoops".to_string())];
        let expected = Err(CommandError::Malformed);

        assert_eq!(expected, Command::from_tokens(&input));
    }

    #[test]
    fn it_parses_echo_commands() {
        let msg = "hello world".to_string();
        let input = vec![
            Token::Array(2),
            Token::SimpleString("echo".to_string()),
            Token::SimpleString(msg.clone()),
        ];
        let bytes: Vec<_> = msg.bytes().collect();
        let expected = Ok((Command::Echo(bytes.into()), 3));

        assert_eq!(expected, Command::from_tokens(&input));
    }
}
