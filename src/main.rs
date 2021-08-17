use std::{fmt::Display, fs::File, io::{BufRead, BufReader}};

use account_manager::AccountManager;
use processor::parse_line;
use tx_amount::TxAmount;

mod account;
mod account_manager;
mod error;
mod processor;
mod transaction;
mod tx_amount;

pub type ClientID   = u16;
pub type TxID       = u32;

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LedgerAction {
    Deposit(TxAmount),
    Withdrawal(TxAmount),
    Dispute,
    Resolve,
    Chargeback,
}

impl Display for LedgerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerAction::Deposit(amount) => write!(f, "deposit ({})", amount),
            LedgerAction::Withdrawal(amount) => write!(f, "withdrawal ({})", amount),
            LedgerAction::Dispute => write!(f, "dispute"),
            LedgerAction::Resolve => write!(f, "resolve"),
            LedgerAction::Chargeback => write!(f, "chargeback"),
        }
    }
}

pub struct LedgerItem {
    client_id:  ClientID,
    tx_id:      TxID,

    action:     LedgerAction,
}

fn main() -> std::io::Result<()> {
    let filename = "transactions.csv";

    let input = File::open(filename)?;

    let reader = BufReader::new(input);
    let mut account_manager = AccountManager::new();

    let mut index = 0;

    let mut lines = reader.lines();

    // Skip header
    lines.next();

    for line in lines {
        if let Ok(line) = line {
            match parse_line(line.as_str()) {
                Ok(item) => {
                    if let Err(e) = account_manager.process(item) {
                        eprintln!("[Line {}] {:?}", index, e);
                    }
                },
                Err(_) => {
                    eprintln!("[Line {}] Could not parse line", index);
                },
            }
        } else {
            eprintln!("Error reading from {}", filename);
            break;
        }

        index += 1;
    }
    
    println!("client, available, held, total, locked");
    for (client_id, account) in account_manager.iter() {
        println!("{}, {}", client_id, account)
    }

    Ok(())
}
