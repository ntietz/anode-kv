use thiserror::Error;

use super::StorageCommand;

#[derive(Error, Debug)]
pub enum TransactionLogError {
    #[error("not implemented")]
    NotImplemented
}


pub trait TransactionLog {
    type Iter: Iterator<Item=StorageCommand>;

    fn record(&self, cmd: StorageCommand) -> Result<(), TransactionLogError>;
    fn compact(&self) -> Result<(), TransactionLogError>;
    fn fsync(&self) -> Result<(), TransactionLogError>;

    fn read(&self) -> Self::Iter;
}

struct NaiveFileBackedTransactionLog {
    // current new writes
    // snapshot

    // <- new new writes
    //

    dirname: String
}

impl NaiveFileBackedTransactionLog {
    fn new(dirname: String) -> Self {



        NaiveFileBackedTransactionLog {
            dirname
        }
    }
}

impl TransactionLog for NaiveFileBackedTransactionLog {
    type Iter = Vec<StorageCommand>::Iter<Item=StorageCommand>;

    fn record(&self, cmd: StorageCommand) -> Result<(), TransactionLogError> {
        Err(TransactionLogError::NotImplemented)
    }

    fn compact(&self) -> Result<(), TransactionLogError> {
        // we can get writes during the compaction
        // the first thing we do with compaction:
        //  - redirect writes to a new file
        //  - start compacting the old file

        Err(TransactionLogError::NotImplemented)
    }

    fn read(&self) -> Self::Iter {
        let items: Vec<StorageCommand> = vec![];
        items.iter()
    }
}

