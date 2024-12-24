use std::collections::HashMap;
use std::env;
use std::sync::{
    atomic::AtomicBool,
    mpsc::{self, Receiver, Sender},
    Arc, Mutex,
};
extern crate csv as ECSV;

use crate::csv::{reader, writer};
use futures::future::join_all;
use processors::txprocessor;
use structs::{
    clients::ClientAccount,
    transaction::{Transaction, TransactionRecord},
};

mod csv;
mod processors;
mod structs;

#[tokio::main]
async fn main() {
    let arguments: Vec<String> = env::args().collect();
    let csv_file = arguments[1].clone();

    // Client records on a HashMap, the key is the client's ID
    let clients: HashMap<u16, ClientAccount> = HashMap::new();
    let clients_ledger = Arc::new(Mutex::new(clients));

    // Tx records on a HashMap, the key is the tx's ID
    let transactions: HashMap<u32, TransactionRecord> = HashMap::new();
    let transactions_ledger = Arc::new(Mutex::new(transactions));

    // Atomic flags to write the client's records to STDOUT
    let start_write = Arc::new(AtomicBool::new(false));
    let start_writer = Arc::clone(&start_write);

    // Channels for the task communication
    let (tx_transactions, rx_transactions): (Sender<Transaction>, Receiver<Transaction>) =
        mpsc::channel();
    let (tx_transactions2, rx_transactions2): (Sender<Transaction>, Receiver<Transaction>) =
        mpsc::channel();

    // Tasks handlers
    let mut handlers = vec![];

    // Reader task
    let tx_clone_reader = tx_transactions.clone();
    handlers.push(tokio::spawn(async {
        reader::read(tx_clone_reader, csv_file).unwrap()
    }));

    // task that will store the Transactions to the HashMap
    let tl_store = Arc::clone(&transactions_ledger);
    let tx_store = tx_transactions2.clone();
    handlers.push(tokio::spawn(async {
        txprocessor::store_transactions(rx_transactions, tx_store, tl_store).unwrap()
    }));

    // task that will process the Transactions and, by the end,
    // enable the writer task
    let tl_process = Arc::clone(&transactions_ledger);
    let cl_process = Arc::clone(&clients_ledger);

    handlers.push(tokio::spawn(async {
        txprocessor::process_transactions(rx_transactions2, tl_process, cl_process, start_write)
            .unwrap()
    }));

    let results = join_all(handlers).await;

    for result in results {
        result.unwrap();
    }

    // By last, writer task that will print the client records to STDOUT
    let handle_writer =
        tokio::spawn(async { writer::write(clients_ledger, start_writer).unwrap() });
    handle_writer.await.unwrap();
}
