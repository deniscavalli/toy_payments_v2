use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::structs::clients::ClientAccount;

/// Writes a SCV to the STDOUT from a HashMap of ClientAccount
/// This function is designed to run in a thread
///
/// # Arguments
///
/// * `clients_ledger` - A reference HashMap of Clients, protected by a Mutex
/// * `start_writing` - Atomic bool in order to start the writing
pub fn write(
    clients_ledger: Arc<Mutex<HashMap<u16, ClientAccount>>>,
    start_writing: Arc<AtomicBool>,
) -> Result<(), Box<dyn Error>> {
    let mut stop = false;
    while !stop && start_writing.load(Ordering::Relaxed) {
        let mut wtr = csv::Writer::from_writer(io::stdout());
        for (_, value) in clients_ledger.lock().unwrap().iter() {
            wtr.serialize(value)?;
        }
        wtr.flush()?;
        stop = true;
    }
    Ok(())
}
