use std::time::Duration;

use tokio::sync::oneshot;

use crate::codec::Token;
use crate::server::Context;
use crate::storage::StorageCommand;

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
                ExecutionResult(vec![Token::BulkString(Some("ECHO".bytes().collect()))])
            }
            Command::Get(key) => {
                let cmd = StorageCommand::Get(key.clone());
                let (tx, rx) = oneshot::channel();
                let res = self
                    .context
                    .storage_queue
                    .send_timeout((cmd, tx), Duration::from_millis(1_000))
                    .await;

                if res.is_err() {
                    return "timeout while sending to storage".into();
                }

                match rx.await {
                    Ok(Ok(None)) => ExecutionResult(vec![Token::BulkString(None)]),
                    Ok(Ok(Some(value))) => ExecutionResult(vec![value.into()]),
                    Ok(Err(_)) => "internal storage error".into(),
                    Err(_) => "no response from storage".into(),
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

                if res.is_err() {
                    return "timeout while sending to storage".into();
                }

                match rx.await {
                    Ok(Ok(None)) => ExecutionResult(vec![Token::SimpleString("OK".to_string())]),
                    Ok(Ok(Some(value))) => ExecutionResult(vec![value.into()]),
                    Ok(Err(_)) => "internal storage error".into(),
                    Err(_) => "no response from storage".into(),
                }
            }
            Command::Unknown(cmd) => {
                return format!("{} is not implemented", cmd).into()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::mpsc;

    use super::*;
    use crate::types::Bytes;

    #[tokio::test]
    async fn it_echoes() {
        let (tx, _rx) = mpsc::channel(1);
        let context = Context::new(tx);
        let cp = CommandProcessor::new(context);

        let cmd = Command::Echo(Bytes(vec![0u8, 1u8, 2u8]));
        let expected = vec![Token::BulkString(Some(vec![0u8, 1u8, 2u8]))];

        let result = cp.execute_command(&cmd).await.0;

        assert_eq!(expected, result);
    }
}
