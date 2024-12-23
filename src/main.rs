use std::collections::HashMap;
use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::sync::Mutex;
extern crate csv as ECSV;

use crate::csv::{reader, writer};
use processors::txprocessor;
use structs::clients::ClientAccount;
use structs::transaction::{Transaction, TransactionRecord};

use std::thread;

mod csv;
mod processors;
mod structs;

fn main() {
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

    // Channels for the threads communication
    let (tx_transactions, rx_transactions): (Sender<Transaction>, Receiver<Transaction>) =
        mpsc::channel();
    let (tx_transactions2, rx_transactions2): (Sender<Transaction>, Receiver<Transaction>) =
        mpsc::channel();

    // Reader thread    
    let tx_clone_reader = tx_transactions.clone();
    let handle_reader = thread::spawn(|| reader::read(tx_clone_reader, csv_file).unwrap());

    // Thread that will store the Transactions to the HashMap
    let tl_store = Arc::clone(&transactions_ledger);
    let tx_store = tx_transactions2.clone();
    let handle_store = thread::spawn(|| {
        txprocessor::store_transactions(rx_transactions, tx_store, tl_store).unwrap()
    });

    // Thread that will process the Transactions and, by the end,
    // enable the writer thread
    let tl_process = Arc::clone(&transactions_ledger);
    let cl_process = Arc::clone(&clients_ledger);
    let handle_process = thread::spawn(|| {
        txprocessor::process_transactions(rx_transactions2, tl_process, cl_process, start_write)
            .unwrap()
    });

    handle_reader.join().unwrap();
    handle_store.join().unwrap();
    handle_process.join().unwrap();

    // By last, writer thread that will print the client records to STDOUT
    let handle_writer = thread::spawn(|| writer::write(clients_ledger, start_writer).unwrap());
    handle_writer.join().unwrap();
}
