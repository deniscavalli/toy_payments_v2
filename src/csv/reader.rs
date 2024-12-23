extern crate csv;

use csv::{ReaderBuilder, Trim};
use std::sync::mpsc::{SendError, Sender};
use thiserror::Error;
use crate::structs::transaction::Transaction;

// CSV Reader Error definition
#[derive(Error, Debug)]
pub enum CSVReaderError {
    #[error("Error reading the input file")]
    ReadingError,
    #[error("Error opening the input file")]
    FileOpeningError(#[from] ECSV::Error),
    #[error("Failed sending the transaction")]
    TxFailError(#[from] SendError<Transaction>)
}

/// Reads a CSV entry from csv_file_path and send it to the Sender
///
/// # Arguments
///
/// * `tx_channel` - A Sender channel that the entries will be sent
///
pub fn read(tx_channel: Sender<Transaction>, csv_file_path: String) -> Result<(), CSVReaderError> {
    let mut rdr = ReaderBuilder::new()
        .trim(Trim::All)
        .from_path(csv_file_path)?;
    for tx in rdr.deserialize() {
        match tx {
            Ok(_) => {
                tx_channel.send(tx?)?;
            }
            Err(_) => {
                return Err(CSVReaderError::ReadingError);
            }
        }
    }
    Ok(())
}
