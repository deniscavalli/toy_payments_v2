use rust_decimal::prelude::*;
use serde::Serialize;

pub type ClientResult = Result<(), String>;

// Client account struct
#[derive(Serialize, Clone, Copy, Debug, Default)]
pub struct ClientAccount {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

// Client account implementation
impl ClientAccount {
    /// Returns a new ClientAccount with the given client id
    ///
    /// # Arguments
    ///
    /// * `client` - Id for the Client
    ///
    /// # Examples
    ///
    /// ```
    /// let client = ClientAccount::new(1);
    /// ```
    pub fn new(client: u16) -> ClientAccount {
        ClientAccount {
            client: client,
            available: Decimal::new(0, 4),
            held: Decimal::new(0, 4),
            total: Decimal::new(0, 4),
            locked: false,
        }
    }

    // Updates total amount of the clinet
    pub fn update_total(&mut self) {
        self.total = self.available + self.held;
    }

    // Make a deposit in the client's account
    // It should not deposit if the account is locked
    pub fn deposit(&mut self, amount: Decimal) -> ClientResult {
        if !self.locked {
            self.available = self.available + amount;
            self.update_total();
        }
        Ok(())
    }

    // Make a withdrawal in the client's account
    // It should not withdrawal if the account is locked or
    // if it doesn't have the necessary funds
    pub fn withdrawal(&mut self, amount: Decimal) -> ClientResult {
        if !self.locked && self.available - amount >= Decimal::new(0, 4) {
            self.available = self.available - amount;
            self.update_total();
        }
        Ok(())
    }

    // Start a dispute in the client's account
    // It should not dispute if the account is locked or
    // if it doesn't have the necessary funds
    pub fn dispute(&mut self, amount: Decimal) -> ClientResult {
        if !self.locked && self.available - amount >= Decimal::new(0, 4) {
            self.available = self.available - amount;
            self.held = self.held + amount;
            self.update_total();
        }
        Ok(())
    }

    // Resolve a dispute in the client's account
    // It should not resolve if the account is locked or
    // if it doesn't have the necessary funds
    pub fn resolve(&mut self, amount: Decimal) -> ClientResult {
        if !self.locked && self.held - amount >= Decimal::new(0, 4) {
            self.available = self.available + amount;
            self.held = self.held - amount;
            self.update_total();
        }
        Ok(())
    }

    // Chargeback an amount from the client's account
    // It should not Chargeback if the account is locked or
    // if it doesn't have the necessary funds
    pub fn chargeback(&mut self, amount: Decimal) -> ClientResult {
        if !self.locked && self.held - amount >= Decimal::new(0, 4) {
            self.held = self.held - amount;
            self.update_total();
            self.locked = true;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_update_total() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.update_total();
        assert_eq!(ca.total, Decimal::new(30, 0));
    }

    #[test]
    fn test_deposit() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.deposit(Decimal::new(50, 0)).unwrap();
        assert_eq!(ca.available, Decimal::from_f32(65.45).unwrap().round_dp(4));
        assert_eq!(ca.total, Decimal::new(80, 0));
    }

    #[test]
    fn test_deposit_locked() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: true,
        };
        ca.deposit(Decimal::new(50, 0)).unwrap();
        assert_eq!(ca.available, Decimal::from_f32(15.45).unwrap().round_dp(4));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_withdrawal() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.withdrawal(Decimal::new(15, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(4500, 4));
        assert_eq!(ca.total, Decimal::new(15, 0));
    }

    #[test]
    fn test_withdrawal_locked() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: true,
        };
        ca.withdrawal(Decimal::new(15, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_withdrawal_insufficient_amount() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.withdrawal(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_dispute() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.dispute(Decimal::new(10, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(0545, 2));
        assert_eq!(ca.held, Decimal::new(2455, 2));
        assert_eq!(ca.total, Decimal::new(30, 0));
    }

    #[test]
    fn test_dispute_locked() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: true,
        };
        ca.dispute(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_dispute_insufficient_amount() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.dispute(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_resolve() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.resolve(Decimal::new(10, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(2545, 2));
        assert_eq!(ca.held, Decimal::new(0455, 2));
        assert_eq!(ca.total, Decimal::new(30, 0));
    }

    #[test]
    fn test_resolve_locked() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: true,
        };
        ca.resolve(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_resolve_insufficient_amount() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.resolve(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_chargeback() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.chargeback(Decimal::new(10, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(0455, 2));
        assert_eq!(ca.total, Decimal::new(20, 0));
        assert_eq!(ca.locked, true);
    }

    #[test]
    fn test_chargeback_locked() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: true,
        };
        ca.chargeback(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }

    #[test]
    fn test_chargeback_insufficient_amount() {
        let mut ca = ClientAccount {
            client: 0,
            available: Decimal::from_f32(15.45).unwrap().round_dp(4),
            held: Decimal::from_f32(14.55).unwrap().round_dp(4),
            total: Decimal::new(0, 4),
            locked: false,
        };
        ca.chargeback(Decimal::new(80, 0)).unwrap();
        assert_eq!(ca.available, Decimal::new(1545, 2));
        assert_eq!(ca.held, Decimal::new(1455, 2));
        assert_eq!(ca.total, Decimal::new(0, 0));
    }
}
