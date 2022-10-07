use std::time::Duration;

use tokio::sync::oneshot;

use crate::codec::Token;
use crate::server::Context;
use crate::storage::{StorageCommand, StorageError};
use crate::types::{Blob, Value};

mod types;
pub use types::{Command, CommandError};

/// CommandProcessor is responsible for taking a group of tokens, executing them,
/// and returning the result.
pub struct CommandProcessor {
    context: Context,
}

#[derive(Debug)]
pub struct ExecutionResult(pub Vec<Token>);

impl IntoIterator for ExecutionResult {
    type Item = Token;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<S: AsRef<str>> From<S> for ExecutionResult {
    fn from(s: S) -> Self {
        ExecutionResult(vec![Token::Error(s.as_ref().to_string())])
    }
}

impl CommandProcessor {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    pub async fn execute_command(&self, command: &Command) -> ExecutionResult {
        match command {
            Command::Echo(t) => ExecutionResult(vec![t.clone().into()]),
            Command::Command => {
                let commands = vec![
                    Token::BulkString(Some("ECHO".bytes().collect())),
                    Token::BulkString(Some("COMMAND".bytes().collect())),
                    Token::BulkString(Some("GET".bytes().collect())),
                    Token::BulkString(Some("SET".bytes().collect())),
                    Token::BulkString(Some("INCR".bytes().collect())),
                    Token::BulkString(Some("DECR".bytes().collect())),
                ];

                let mut resp = vec![Token::Array(commands.len() as i64)];
                resp.extend_from_slice(&commands[..]);

                ExecutionResult(resp)
            }
            Command::Get(key) => {
                self.execute_command_helper(StorageCommand::Get(key.clone()), |res| match res {
                    Ok(Ok(None)) => ExecutionResult(vec![Token::BulkString(None)]),
                    Ok(Ok(Some(value))) => ExecutionResult(value_to_tokens(value)),
                    Ok(Err(_)) => "internal storage error".into(),
                    Err(_) => "no response from storage".into(),
                })
                .await
            }
            Command::Set(key, value) => {
                self.execute_command_helper(
                    StorageCommand::Set(key.clone(), Value::Blob(value.clone())),
                    |res| match res {
                        Ok(Ok(None)) => {
                            ExecutionResult(vec![Token::SimpleString("OK".to_string())])
                        }
                        Ok(Ok(Some(value))) => ExecutionResult(value_to_tokens(value)),
                        Ok(Err(_)) => "internal storage error".into(),
                        Err(_) => "no response from storage".into(),
                    },
                )
                .await
            }
            Command::Incr(key) => {
                self.execute_command_helper(StorageCommand::Incr(key.clone()), |res| match res {
                    Ok(Ok(Some(value))) => ExecutionResult(value_to_tokens(value)),
                    Ok(Ok(None)) => "invalid response from storage".into(),
                    Ok(Err(err)) => storage_error_to_string(err).into(),
                    Err(_) => "no response from storage".into(),
                })
                .await
            }
            Command::Decr(key) => {
                self.execute_command_helper(StorageCommand::Decr(key.clone()), |res| match res {
                    Ok(Ok(Some(value))) => ExecutionResult(value_to_tokens(value)),
                    Ok(Ok(None)) => "invalid response from storage".into(),
                    Ok(Err(err)) => storage_error_to_string(err).into(),
                    Err(_) => "no response from storage".into(),
                })
                .await
            }
            Command::Unknown(cmd) => format!("{} is not implemented", cmd).into(),
        }
    }

    async fn execute_command_helper(
        &self,
        cmd: StorageCommand,
        f: impl FnOnce(
            Result<Result<Option<Value>, StorageError>, tokio::sync::oneshot::error::RecvError>,
        ) -> ExecutionResult,
    ) -> ExecutionResult {
        let (tx, rx) = oneshot::channel();
        let res = self
            .context
            .storage_queue
            .send_timeout((cmd, tx), Duration::from_millis(1_000))
            .await;

        if res.is_err() {
            return "timeout while sending to storage".into();
        }

        f(rx.await)
    }
}

fn value_to_tokens(value: Value) -> Vec<Token> {
    match value {
        Value::Blob(b) => vec![b.into()],
        Value::Int(i) => {
            let b = i.to_string().into_bytes();
            vec![Blob(b).into()]
        }
    }
}

fn storage_error_to_string(error: StorageError) -> &'static str {
    match error {
        StorageError::NotAnInteger => {
            "WRONGTYPE Operation against a key holding the wrong kind of value"
        }
        StorageError::Overflow => "ERR increment or decrement would overflow",
        StorageError::Failed(_) => "ERR unknown storage failure",
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;
    use crate::types::Blob;

    #[tokio::test]
    async fn it_echoes() {
        let (tx, _rx) = mpsc::channel(1);
        let context = Context::new(tx);
        let cp = CommandProcessor::new(context);

        let cmd = Command::Echo(Blob(vec![0u8, 1u8, 2u8]));
        let expected = vec![Token::BulkString(Some(vec![0u8, 1u8, 2u8]))];

        let result = cp.execute_command(&cmd).await.0;

        assert_eq!(expected, result);
    }
}
