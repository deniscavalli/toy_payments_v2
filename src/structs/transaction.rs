use rust_decimal::prelude::*;
use serde::Deserialize;

// Transaction struct
#[derive(Clone, Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    amount: Option<f32>,
}

// Transaction implementation
impl Transaction {
    pub fn client(&self) -> u16 {
        self.client
    }

    pub fn tx(&self) -> u32 {
        self.tx
    }

    pub fn tx_type(self) -> String {
        self.tx_type
    }

    pub fn amount(&self) -> Option<f32> {
        self.amount
    }
}

// Transaction record struct
// This struct is for internal storage and calculations
// Transaction should be parsed into this stuct for use
#[derive(Clone, Copy, Debug)]
pub struct TransactionRecord {
    amount: Decimal,
    client: u16,
    disputed: bool,
}

// Transaction record implementation
impl TransactionRecord {
    pub fn amount(self) -> Decimal {
        self.amount
    }

    pub fn client(self) -> u16 {
        self.client
    }

    pub fn disputed(self) -> bool {
        self.disputed
    }

    pub fn dispute(&mut self) {
        self.disputed = true;
    }

    pub fn resolve(&mut self) {
        self.disputed = false;
    }
}

// From Trait implementation, to correct parse from Transaction
impl From<&Transaction> for TransactionRecord {
    fn from(t: &Transaction) -> Self {
        let am: f32 = t.amount().unwrap_or(0.0000);
        TransactionRecord {
            client: t.client,
            disputed: false,
            amount: Decimal::from_f32(am)
                .unwrap_or(Decimal::new(0, 4))
                .round_dp(4),
        }
    }
}

// Unit tests
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_from() {
        let tr: TransactionRecord = TransactionRecord::from(&Transaction {
            client: 1,
            tx: 2,
            tx_type: "deposit".to_string(),
            amount: Some(42.00),
        });
        assert_eq!(tr.client, tr.client());
        assert_eq!(tr.disputed, false);
        assert_eq!(tr.amount(), tr.amount());
    }
}
