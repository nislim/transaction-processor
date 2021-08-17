use std::fmt::Display;

use processor::parse_line;
use tokio::{fs::File, io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter}, sync::mpsc::channel};
use tx_amount::TxAmount;

use crate::account_manager::AccountManagerLoadbalancer;

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

#[derive(Debug)]
pub struct LedgerItem {
    client_id:  ClientID,
    tx_id:      TxID,

    action:     LedgerAction,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let filename = "transactions.csv";

    let input = File::open(filename).await?;

    let reader = BufReader::new(input);
    let account_manager = AccountManagerLoadbalancer::spawn(8);

    let mut index = 0;

    let mut lines = reader.lines();

    // Skip header
    lines.next_line().await.unwrap();

    while let Some(line) = lines.next_line().await? {
        match parse_line(line.as_str()) {
            Ok(item) => {
                account_manager.process(item).await
            },
            Err(_) => {
                eprintln!("[Line {}] Could not parse line", index);
            },
        }

        index += 1;
    }

    let (sender, mut receiver) = channel(128);
    
    account_manager.dump(sender).await;

    let stdout = tokio::io::stdout();
    let mut writer = BufWriter::new(stdout);

    let header = b"client, available, held, total, locked\n";
    
    writer.write_all(header).await?;
    
    while let Some((client_id, available, held, total, locked)) = receiver.recv().await {
        writer.write_all(client_id.to_string().as_bytes()).await?;
        writer.write_all(b", ").await?;

        writer.write_all(available.to_string().as_bytes()).await?;
        writer.write_all(b", ").await?;

        writer.write_all(held.to_string().as_bytes()).await?;
        writer.write_all(b", ").await?;

        writer.write_all(total.to_string().as_bytes()).await?;
        writer.write_all(b", ").await?;

        writer.write_all(locked.to_string().as_bytes()).await?;
        writer.write_all(b"\n").await?;
    }

    account_manager.stop().await;
    account_manager.join().await;

    writer.flush().await.unwrap();

    Ok(())
}
