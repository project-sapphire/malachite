extern crate rand;
use self::rand::Rng;


#[derive(Debug, Clone)]
pub enum WithdrawError {
    Balance(f64)
}

#[derive(Debug)]
pub struct Wallet {
    currency: String,
    balance: f64,
    address: String,
}

impl Wallet {
    pub fn new(currency: String) -> Wallet {
        Wallet {
            currency: currency,
            balance: 0.0,
            address: Self::create_address(),
        }
    }

    pub fn new_from_saved(currency: String, balance: f64, address: String) -> Wallet {
        Wallet {
            currency: currency,
            balance: balance,
            address: address,
        }
    }

    fn create_address() -> String {
        let mut rng = rand::StdRng::new().unwrap();
        rng.gen_ascii_chars().take(16).collect()
    }

    pub fn deposit(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn withdraw(&mut self, amount: f64) -> Result<f64, WithdrawError> {
        if self.balance < amount {
            return Err(WithdrawError::Balance(self.balance))
        }

        self.balance -= amount;
        Ok(amount)
    }

    pub fn get_balance(&self) -> f64 {
        self.balance
    }

    pub fn get_currency(&self) -> String {
        self.currency.clone()
    }

    pub fn get_address(&self) -> String {
        self.address.clone()
    }
}
