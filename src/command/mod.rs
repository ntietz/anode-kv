use std::time::Duration;

use tokio::sync::oneshot;

use crate::codec::Token;
use crate::server::Context;
use crate::storage::StorageCommand;

mod command;
pub use command::{Command, CommandError};

/// CommandProcessor is responsible for taking a group of tokens, executing them,
/// and returning the result.
pub struct CommandProcessor {
    context: Context,
}

impl CommandProcessor {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn execute_command(&self, command: &Command) -> Vec<Token> {
        match command {
            Command::Echo(t) => vec![Token::BulkString(Some(t.clone()))],
            Command::Command => vec![Token::BulkString(Some("ECHO".bytes().collect()))],
            Command::Get(key) => {
                let cmd = StorageCommand::Get(key.clone());
                let (tx, rx) = oneshot::channel();
                let res = self
                    .context
                    .storage_queue
                    .send_timeout((cmd, tx), Duration::from_millis(1_000))
                    .await;

                if let Err(_) = res {
                    return error_response("timeout while sending to storage".to_string());
                }

                match rx.await {
                    Ok(Ok(None)) => vec![Token::BulkString(None)],
                    Ok(Ok(Some(value))) => vec![Token::BulkString(Some(value))],
                    Ok(Err(_)) => error_response("internal storage error".to_string()),
                    Err(_) => error_response("no response from storage".to_string()),
                }
            }
            Command::Set(key, value) => {
                let cmd = StorageCommand::Set(key.clone(), value.clone());
                let (tx, rx) = oneshot::channel();
                let res = self
                    .context
                    .storage_queue
                    .send_timeout((cmd, tx), Duration::from_millis(1_000))
                    .await;

                if let Err(_) = res {
                    return error_response("timeout while sending to storage".to_string());
                }

                match rx.await {
                    Ok(Ok(None)) => vec![Token::SimpleString("OK".to_string())],
                    Ok(Ok(Some(value))) => vec![Token::BulkString(Some(value))],
                    Ok(Err(_)) => error_response("internal storage error".to_string()),
                    Err(_) => error_response("no response from storage".to_string()),
                }
            }
        }
    }
}

fn error_response(msg: String) -> Vec<Token> {
    vec![Token::Error(msg)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_echoes() {
        let cp = CommandProcessor::new(Context::dummy());

        let cmd = Command::Echo(vec![0u8, 1u8, 2u8]);
        let expected = vec![Token::BulkString(Some(vec![0u8, 1u8, 2u8]))];

        let result = cp.execute_command(&cmd).await;

        assert_eq!(expected, result);
    }
}
