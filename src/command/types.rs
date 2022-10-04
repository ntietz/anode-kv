use thiserror::Error;

use crate::codec::Token;
use crate::types::{Blob, Key, Value};

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Echo(Value),

    Command,

    Get(Key),
    Set(Key, Value),

    Decr(Key),
    Incr(Key),

    Unknown(String),
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum CommandError {
    #[error("insufficient tokens")]
    InsufficientTokens,

    #[error("malformed input")]
    Malformed,
}

impl Command {
    pub fn from_tokens(tokens: &[Token]) -> Result<(Command, usize), CommandError> {
        if tokens.is_empty() {
            return Err(CommandError::InsufficientTokens);
        }

        let (length, cmd) = get_command(tokens)?;

        match cmd.as_str() {
            "ECHO" => {
                validate_length(length, ECHO_LENGTH)?;
                let reply_token = string_token_as_bytes(tokens.get(2))?;
                Ok((Command::Echo(reply_token), ECHO_LENGTH + 1))
            }
            "COMMAND" => {
                validate_length(length, COMMAND_LENGTH)?;
                Ok((Command::Command, COMMAND_LENGTH + 1))
            }
            "GET" => {
                validate_length(length, GET_LENGTH)?;
                let key = string_token_as_bytes(tokens.get(2))?;

                Ok((Command::Get(key), GET_LENGTH + 1))
            }
            "SET" => {
                validate_length(length, SET_LENGTH)?;
                let key = string_token_as_bytes(tokens.get(2))?;
                let value = string_token_as_bytes(tokens.get(3))?;

                Ok((Command::Set(key, value), SET_LENGTH + 1))
            }
            unk => Ok((Command::Unknown(unk.to_string()), length + 1)),
        }
    }
}

const ECHO_LENGTH: usize = 2;
const COMMAND_LENGTH: usize = 1;
const GET_LENGTH: usize = 2;
const SET_LENGTH: usize = 3;

fn get_command(tokens: &[Token]) -> Result<(usize, String), CommandError> {
    let length = match tokens.get(0) {
        Some(Token::Array(l)) if (*l) > 0 => (*l) as usize,
        _ => return Err(CommandError::Malformed),
    };

    if tokens.len() - 1 < length {
        return Err(CommandError::InsufficientTokens);
    }

    let cmd = match tokens.get(1) {
        Some(Token::SimpleString(c)) => c.to_uppercase(),
        Some(Token::BulkString(Some(c))) => match std::str::from_utf8(c) {
            Ok(s) => s.to_uppercase(),
            Err(_) => return Err(CommandError::Malformed),
        },
        _ => return Err(CommandError::Malformed),
    };

    Ok((length, cmd))
}

fn string_token_as_bytes(token: Option<&Token>) -> Result<Blob, CommandError> {
    match token {
        Some(Token::SimpleString(s)) => Ok(s.bytes().collect::<Vec<u8>>().into()),
        Some(Token::BulkString(Some(s))) => Ok(s.clone().into()),
        _ => Err(CommandError::Malformed),
    }
}

fn validate_length(length: usize, expected_length: usize) -> Result<(), CommandError> {
    if length != expected_length {
        return Err(CommandError::Malformed);
    }
    Ok(())
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
