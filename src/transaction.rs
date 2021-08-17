use std::fmt::{Display};

use crate::*;

pub struct TransactionDelta {
    pub available: TxAmount,
    pub held:      TxAmount,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum TransactionState {
    New,
    Disputed,
    Resolved,
    Chargeback,
}

pub struct Transaction {
    amount: TxAmount,
    state:  TransactionState,
}

pub enum TransactionError {
    NegativeDeposit,
    NegativeWithdrawal,
    InvalidTransactionStateTransition(TransactionState, TransactionState),
}

impl Transaction {

    pub fn deposit(amount: TxAmount) -> Result<(Self, TransactionDelta), TransactionError> {
        if amount < FpIsize::zero() {
            Err(TransactionError::NegativeDeposit)
        } else {
            Ok((
                Transaction {
                    amount,
                    state: TransactionState::New,
                },
                TransactionDelta {
                    available:  amount,
                    held:       FpIsize::zero(),
                }
            ))
        }
    }

    pub fn withdraw(amount: TxAmount) -> Result<(Self, TransactionDelta), TransactionError> {
        if amount < FpIsize::zero() {
            Err(TransactionError::NegativeWithdrawal)
        } else {
            let amount = -amount;

            Ok((
                Transaction {
                    amount,
                    state: TransactionState::New,
                },
                TransactionDelta {
                    available:  amount,
                    held:       FpIsize::zero(),
                }
            ))
        }
    }

    pub fn dispute(&mut self) -> Result<TransactionDelta, TransactionError> {
        if TransactionState::New == self.state {
            self.state = TransactionState::Disputed;

            Ok(TransactionDelta {
                available: -self.amount,
                held:       self.amount
            })
        } else {
            Err(TransactionError::InvalidTransactionStateTransition(self.state, TransactionState::Disputed))
        }
    }

    pub fn resolve(&mut self) -> Result<TransactionDelta, TransactionError> {
        if TransactionState::Disputed == self.state {
            self.state = TransactionState::Resolved;

            Ok(TransactionDelta {
                available:  self.amount,
                held:      -self.amount
            })
        } else {
            Err(TransactionError::InvalidTransactionStateTransition(self.state, TransactionState::Resolved))
        }
    }

    pub fn chargeback(&mut self) -> Result<TransactionDelta, TransactionError> {
        if TransactionState::Disputed == self.state {
            self.state = TransactionState::Chargeback;

            Ok(TransactionDelta {
                available:  FpIsize::zero(),
                held:      -self.amount
            })
        } else {
            Err(TransactionError::InvalidTransactionStateTransition(self.state, TransactionState::Chargeback))
        }
    }
}

impl Display for TransactionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionState::New => write!(f, "New"),
            TransactionState::Disputed => write!(f, "Disputed"),
            TransactionState::Resolved => write!(f, "Resolved"),
            TransactionState::Chargeback => write!(f, "Chargeback"),
        }
    }
}