#[macro_use]
extern crate log;
extern crate simplelog;
extern crate prism;
extern crate zmq;

mod wallet;

use std::collections::HashMap;

use prism::Message;

use wallet::{Wallet, WithdrawError};


type WalletCollection = HashMap<String, Wallet>;

fn log_balance(wallets: &WalletCollection) {
    for (_, ref wallet) in wallets {
        info!("- {}: {} {}", wallet.get_address(), wallet.get_currency().to_uppercase(), wallet.get_balance());
    }
}

fn get_or_insert_wallet<'a>(wallets: &'a mut WalletCollection, currency: &str) -> &'a mut Wallet {
    if !wallets.contains_key(currency) {
        wallets.insert(currency.to_string(), Wallet::new(currency.to_string()));
    }

    wallets.get_mut(currency).unwrap()
}

fn get_wallet_address(wallets: &mut WalletCollection, currency: &str) -> String {
    get_or_insert_wallet(wallets, currency).get_address()
}

fn main() {
    simplelog::TermLogger::init(log::LogLevelFilter::Trace, simplelog::Config::default()).expect("failed to start logger");

    let mut wallets: HashMap<String, Wallet> = HashMap::new();

    // TODO(deox): load saved wallets
    let mut btc_wallet = Wallet::new("btc".to_string());
    btc_wallet.deposit(0.5);

    wallets.insert(btc_wallet.get_currency(), btc_wallet);
    wallets.insert("eth".to_string(), Wallet::new("eth".to_string()));
    wallets.insert("xrp".to_string(), Wallet::new("xrp".to_string()));

    log_balance(&wallets);

    // ZMQ setup
    let context = zmq::Context::new();
    let requests = context.socket(zmq::REP).unwrap();
    let backdoor = context.socket(zmq::REQ).unwrap();

    requests.bind("tcp://*:1340").unwrap();
    backdoor.connect("tcp://127.0.0.1:1339").unwrap();

    loop {
        match prism::WalletRequest::receive(&requests, 0) {
            Ok(Some(request)) => {
                info!("Received: {:?}", request);
                match request.query {
                    prism::WalletQuery::Receive => requests.send_str(&get_wallet_address(&mut wallets, &request.currency), 0).unwrap(),
                    prism::WalletQuery::Pay(amount, address) => {
                        info!("Paying invoice of {} {} to {}", request.currency, amount, address);

                        if get_or_insert_wallet(&mut wallets, &request.currency).withdraw(amount).is_err() {
                            warn!("Insufficient balance!");
                            requests.send_str("insufficient balance", 0).unwrap();
                            continue;
                        }
                        
                        // pay amount through secret backdoor
                        backdoor.send_str(&address, 0).unwrap();
                        let resulting_currency = backdoor.recv_string(0).unwrap().unwrap();
                        let resulting_amount = f64::receive(&backdoor, 0).unwrap().unwrap();
                        info!("Exchange resulted in {} {}", resulting_currency.to_lowercase(), resulting_amount);
                        get_or_insert_wallet(&mut wallets, &resulting_currency).deposit(resulting_amount);

                        requests.send_str("ok", 0).unwrap();

                        log_balance(&wallets);
                    },
                    prism::WalletQuery::Currencies => wallets.keys().map(|x|x.clone()).collect::<Vec<String>>().send(&requests, 0).unwrap(),
                    prism::WalletQuery::Balance => get_or_insert_wallet(&mut wallets, &request.currency).get_balance().send(&requests, 0).unwrap(),
                };
            },
            x => {
                error!("{:?}", x);
                break;
            }
        }
    }

    error!("Ouch!");
}
