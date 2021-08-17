use std::fmt::Debug;
use crate::{ClientID, LedgerAction, LedgerItem, TxID, transaction::{TransactionError, TransactionState}};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum ProcessorError {
    NegativeDeposit(ClientID, TxID),
    NegativeWithdrawal(ClientID, TxID),
    InvalidTransactionStateTransition(ClientID, TxID, TransactionState, TransactionState),
    DuplicateTransaction(ClientID, TxID),
    MissingTransaction(ClientID, TxID, LedgerAction),
    InsufficientFunds(ClientID, TxID),
    LockedAccount(ClientID, TxID),
}

impl From<(&LedgerItem, TransactionError)> for ProcessorError {
    fn from((item, err): (&LedgerItem, TransactionError)) -> Self {
        match err {
            TransactionError::NegativeDeposit => ProcessorError::NegativeDeposit(item.client_id, item.tx_id),
            TransactionError::NegativeWithdrawal => ProcessorError::NegativeWithdrawal(item.client_id, item.tx_id),
            TransactionError::InvalidTransactionStateTransition(src, dst) => ProcessorError::InvalidTransactionStateTransition(item.client_id, item.tx_id, src, dst),
        }
    }
}

impl Debug for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessorError::NegativeDeposit(client_id, tx_id) => 
                write!(f, "[Client {}] Transaction {} (Deposit) has a negative value", client_id, tx_id),
            ProcessorError::NegativeWithdrawal(client_id, tx_id) =>
                write!(f, "[Client {}]  Transaction {} (Withdrawal) has a negative value", client_id, tx_id),
            ProcessorError::InvalidTransactionStateTransition(client_id, tx_id, orig, new) =>
                write!(f, "[Client {}]  Transition {} from state {} to {} is not possible", client_id, tx_id, orig, new),
            ProcessorError::DuplicateTransaction(client_id, tx_id) =>
                write!(f, "[Client {}] Tried to add a duplicate transaction with id {}", client_id, tx_id),
            ProcessorError::MissingTransaction(client_id, tx_id, action) =>
                write!(f, "[Client {}] Tried to {} transaction with id {} but transaction is not found", client_id, action, tx_id),
            ProcessorError::InsufficientFunds(client_id, tx_id) =>
                write!(f, "[Client {}] Insufficient funds to process transaction {}", client_id, tx_id),
            ProcessorError::LockedAccount(client_id, tx_id) =>
                write!(f, "[Client {}] Cannot process transaction {} because the account is locked", client_id, tx_id),
        }
    }
}