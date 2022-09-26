use crate::codec::Token;

mod command;
pub use command::{Command, CommandError};

/// CommandProcessor is responsible for taking a group of tokens, executing them,
/// and returning the result.
pub struct CommandProcessor {}

impl CommandProcessor {
    pub fn execute_command(&self, command: &Command) -> Vec<Token> {
        match command {
            Command::Echo(t) => vec![Token::BulkString(Some(t.clone()))],
            Command::Command => vec![Token::BulkString(Some("ECHO".bytes().collect()))],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_echoes() {
        let cp = CommandProcessor {};

        let cmd = Command::Echo(vec![0u8, 1u8, 2u8]);
        let expected = vec![Token::BulkString(Some(vec![0u8, 1u8, 2u8]))];

        assert_eq!(expected, cp.execute_command(&cmd));
    }
}
