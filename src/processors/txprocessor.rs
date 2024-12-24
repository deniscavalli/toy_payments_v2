use crate::structs::{
    clients::ClientAccount,
    transaction::{Transaction, TransactionRecord},
};
use rust_decimal::prelude::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use thiserror::Error;

// TX Processor Error definition
#[derive(Error, Debug)]
pub enum TXProcessError {
    #[error("Invalid Transaction")]
    InvalidTxType,
}

/// Parse the Transactinos to Transaction Records
/// and add it to a internal use HashMap holding all transactions that
/// can be disputed (Deposit or Withdrawals)
/// and send forward the other transactions
/// This functions is designed to run in a thread.
///
/// # Arguments
///
/// * `rx_channel` - Receiver channel that will receive the Transactions read
/// * `tx_channel` - Sender channel where the Transactions will be send
/// * `tx_ledger` - Transaction HashMap that holds deposit and withdrawals
/// the transaction ID is the key for the Transaction record associated
pub fn store_transactions(
    rx_channel: Receiver<Transaction>,
    tx_channel: Sender<Transaction>,
    tx_ledger: Arc<Mutex<HashMap<u32, TransactionRecord>>>,
) -> Result<(), TXProcessError> {
    // Number of retries before finish the thread
    let mut retry: u32 = 100000;
    let mut stop = false;
    while !stop {
        // Tries to receive a Transaction
        if let Ok(transaction) = rx_channel.try_recv() {
            let tx_clone = transaction.clone();
            match transaction.tx_type().as_str() {
                "deposit" | "withdrawal" => {
                    tx_ledger
                        .lock()
                        .unwrap()
                        .insert(tx_clone.tx(), TransactionRecord::from(&tx_clone));
                    tx_channel.send(tx_clone).unwrap()
                }
                "dispute" | "resolve" | "chargeback" => {
                    tx_channel.send(tx_clone).unwrap();
                }
                _ => return Err(TXProcessError::InvalidTxType),
            }
        } else {
            // If no message is received, try again
            retry = retry - 1;
            if retry == 0 {
                stop = true;
            }
        }
    }
    Ok(())
}

/// Process the transactions, performing the transaction actions, by type.
/// After it finishes runs, it sets the start_writing flag to true,
/// starting the writer thread.
/// This function is designed to run in a thread.
///
/// # Arguments
///
/// * `rx_channel` - Receiver channel that will receive the Transactions read
/// * `tx_ledger` - Transaction HashMap that holds deposit and withdrawals
/// the transaction ID is the key for the Transaction record associated
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `start_writing` - Boolean that starts the writing thread
pub fn process_transactions(
    rx_channel: Receiver<Transaction>,
    tx_ledger: Arc<Mutex<HashMap<u32, TransactionRecord>>>,
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    start_writing: Arc<AtomicBool>,
) -> Result<(), TXProcessError> {
    // Number of retries before finish the thread
    let mut retry: u32 = 100000;
    let mut stop = false;
    while !stop {
        // Tries to receive a Transaction
        if let Ok(transaction) = rx_channel.try_recv() {
            let tx_clone = transaction.clone();
            match tx_clone.tx_type().as_ref() {
                "deposit" => {
                    deposit(
                        Arc::clone(&client_ledger),
                        transaction.client(),
                        Decimal::from_f32(transaction.amount().unwrap_or(0.000))
                            .unwrap_or(Decimal::new(0, 4))
                            .round_dp(4),
                    )
                    .unwrap();
                }
                "withdrawal" => {
                    withdrawal(
                        Arc::clone(&client_ledger),
                        transaction.client(),
                        Decimal::from_f32(transaction.amount().unwrap_or(0.000))
                            .unwrap_or(Decimal::new(0, 4))
                            .round_dp(4),
                    )
                    .unwrap();
                }
                "dispute" => {
                    dispute(
                        Arc::clone(&client_ledger),
                        Arc::clone(&tx_ledger),
                        transaction.tx(),
                        transaction.client(),
                    )
                    .unwrap();
                }
                "resolve" => {
                    resolve(
                        Arc::clone(&client_ledger),
                        Arc::clone(&tx_ledger),
                        transaction.tx(),
                        transaction.client(),
                    )
                    .unwrap();
                }
                "chargeback" => {
                    chargeback(
                        Arc::clone(&client_ledger),
                        Arc::clone(&tx_ledger),
                        transaction.tx(),
                        transaction.client(),
                    )
                    .unwrap();
                }
                _ => return Err(TXProcessError::InvalidTxType),
            }
        } else {
            // If no message is received, try again
            retry = retry - 1;
            if retry == 0 {
                stop = true;
            }
        }
    }
    // Sets the flag to start writing thread.
    start_writing.store(true, Ordering::Relaxed);
    Ok(())
}

/// Deposit action. If the client is not registered, it creates a new entry.
///
/// # Arguments
///
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `client` - Client id to perform the action
/// * `amount` - Amount to be deposited
fn deposit(
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    client: u16,
    amount: Decimal,
) -> Result<(), TXProcessError> {
    let mut cl = client_ledger.lock().unwrap();
    if let Some(client_record) = cl.get_mut(&client) {
        client_record.deposit(amount).unwrap();
    } else {
        let mut new_client = ClientAccount::new(client);
        new_client.deposit(amount).unwrap();
        cl.insert(client, new_client);
    }
    Ok(())
}

/// Withdrawal action. If the client is not registered, it creates a new entry.
///
/// # Arguments
///
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `client` - Client id to perform the action
/// * `amount` - Amount to be withdrawed
fn withdrawal(
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    client: u16,
    amount: Decimal,
) -> Result<(), TXProcessError> {
    let mut cl = client_ledger.lock().unwrap();
    if let Some(client_record) = cl.get_mut(&client) {
        client_record.withdrawal(amount).unwrap();
    } else {
        let new_client = ClientAccount::new(client);
        cl.insert(client, new_client);
    }

    Ok(())
}

/// Dispute action. If there is a Transaction with the designed ID to be disputed,
/// with the righ client ID, it will be disputed.
///
/// # Arguments
///
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `tx_ledger` - Transaction HashMap that holds deposit and withdrawals
/// the transaction ID is the key for the Transaction record associated
/// * `tx_id` - Transaction ID to look for
/// * `client` - Client id to perform the action
fn dispute(
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    tx_ledger: Arc<Mutex<HashMap<u32, TransactionRecord>>>,
    tx_id: u32,
    client: u16,
) -> Result<(), TXProcessError> {
    if let Some(transaction) = tx_ledger.lock().unwrap().get_mut(&tx_id) {
        if transaction.client() == client {
            let mut cl = client_ledger.lock().unwrap();
            if let Some(client_record) = cl.get_mut(&client) {
                client_record.dispute(transaction.amount()).unwrap();
                transaction.dispute();
            }
        }
    }

    Ok(())
}

/// Resolve action. If there is a Transaction with the designed ID to be disputed
/// with the righ client ID and is under a dispute, it will be resolved.
///
/// # Arguments
///
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `tx_ledger` - Transaction HashMap that holds deposit and withdrawals
/// the transaction ID is the key for the Transaction record associated
/// * `tx_id` - Transaction ID to look for
/// * `client` - Client id to perform the action
fn resolve(
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    tx_ledger: Arc<Mutex<HashMap<u32, TransactionRecord>>>,
    tx_id: u32,
    client: u16,
) -> Result<(), TXProcessError> {
    if let Some(transaction) = tx_ledger.lock().unwrap().get_mut(&tx_id) {
        if transaction.disputed() && transaction.client() == client {
            let mut cl = client_ledger.lock().unwrap();
            if let Some(client_record) = cl.get_mut(&client) {
                client_record.resolve(transaction.amount()).unwrap();
                transaction.resolve();
            }
        }
    }

    Ok(())
}

/// Chargeback action. If there is a Transaction with the designed ID to be disputed
/// with the righ client ID and is under a dispute, it will be charged back.
/// But the client will be locked.
///
/// # Arguments
///
/// * `client_ledger` - ClientAccount HashMap that holds clients balance and status,
/// the client ID is the key for the ClientAccount associated
/// * `tx_ledger` - Transaction HashMap that holds deposit and withdrawals
/// the transaction ID is the key for the Transaction record associated
/// * `tx_id` - Transaction ID to look for
/// * `amount` - Amount to be deposited
fn chargeback(
    client_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    tx_ledger: Arc<Mutex<HashMap<u32, TransactionRecord>>>,
    tx_id: u32,
    client: u16,
) -> Result<(), TXProcessError> {
    if let Some(transaction) = tx_ledger.lock().unwrap().get_mut(&tx_id) {
        if transaction.disputed() && transaction.client() == client {
            let mut cl = client_ledger.lock().unwrap();
            if let Some(client_record) = cl.get_mut(&client) {
                client_record.chargeback(transaction.amount()).unwrap();
                transaction.resolve();
            }
        }
    }

    Ok(())
}
