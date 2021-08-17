use std::{collections::{BTreeMap, btree_map::Iter}, fmt::Display};

use crate::error::ProcessorError;

use super:: {
    TxAmount, TxID, LedgerAction, LedgerItem,
    transaction::{Transaction, TransactionDelta},
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum AccountState {
    Active,
    Locked,
}

pub struct Account {
    state:      AccountState,
    available:  TxAmount,
    held:       TxAmount,

    transactions:   BTreeMap<TxID, Transaction>    
}

impl Account {

    pub fn new() -> Self {
        Account {
            state:          AccountState::Active,
            available:      TxAmount::zero(),
            held:           TxAmount::zero(),

            transactions:   BTreeMap::new(),
        }
    }

    fn apply_delta_unchecked(&mut self, delta: TransactionDelta) {
        self.available  += delta.available;
        self.held       += delta.held;
    }

    fn apply_delta(&mut self, delta: TransactionDelta, item: &LedgerItem) -> Result<(), ProcessorError> {
        if self.available + delta.available < TxAmount::zero() {
            Err(ProcessorError::InsufficientFunds(item.client_id, item.tx_id))
        } else {
            self.apply_delta_unchecked(delta);

            Ok(())
        }
    }

    fn lock(&mut self) {
        self.state = AccountState::Locked;
    }

    fn process_internal(&mut self, item: LedgerItem) -> Result<(), ProcessorError> {
        if let Some(transaction) = self.transactions.get_mut(&item.tx_id) {
            let delta = match item.action {
                LedgerAction::Dispute => transaction.dispute().map_err(|e| ProcessorError::from((&item, e))),
                LedgerAction::Resolve => transaction.resolve().map_err(|e| ProcessorError::from((&item, e))),
                LedgerAction::Chargeback => transaction.chargeback().map_err(|e| ProcessorError::from((&item, e))),
                _ => Err(ProcessorError::DuplicateTransaction(item.client_id, item.tx_id)),
            }?;

            if LedgerAction::Chargeback == item.action {
                self.lock();
            }

            self.apply_delta_unchecked(delta);

            Ok(())
        } else {
            let transaction = match item.action {
                LedgerAction::Deposit(amount) => {
                    let (transaction, delta) = Transaction::deposit(amount).map_err(|e| ProcessorError::from((&item, e)))?;
                    
                    self.apply_delta_unchecked(delta);

                    Ok(transaction)
                },
                LedgerAction::Withdrawal(amount) => {
                    let (transaction, delta) = Transaction::withdraw(amount).map_err(|e| ProcessorError::from((&item, e)))?;

                    self.apply_delta(delta, &item)?;

                    Ok(transaction)
                },
                _ => Err(ProcessorError::MissingTransaction(item.client_id, item.tx_id, item.action)),
            }?;

            self.transactions.insert(item.tx_id, transaction);

            Ok(())
        }
    }

    pub fn process(&mut self, item: LedgerItem) -> Result<(), ProcessorError> {
        if AccountState::Locked == self.state {
            Err(ProcessorError::LockedAccount(item.client_id, item.tx_id))
        } else {
            self.process_internal(item)
        }
    }

    pub fn is_locked(&self) -> bool {
        AccountState::Locked == self.state
    }

    pub fn is_active(&self) -> bool {
        AccountState::Active == self.state
    }


    pub fn available(&self) -> TxAmount {
        self.available
    }

    pub fn held(&self) -> TxAmount {
        self.held
    }

    pub fn total(&self) -> TxAmount {
        self.available + self.held
    }

    pub fn transactions(&self) -> Iter<TxID, Transaction> {
        self.transactions.iter()
    }

}

impl Display for Account {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let locked = if self.is_locked() {
            "true"
        } else {
            "false"
        };
    
        write!(f, "{}, {}, {}, {}",
            self.available(), self.held(), self.total(), locked,
        )
    }
}

#[cfg(test)]
mod test {
    

    use crate::{LedgerAction, LedgerItem, TxAmount, error::ProcessorError};

    use super::Account;

    fn setup_account(_amount: TxAmount) -> Account {
        let mut account = Account::new();

        let deposit = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Deposit(TxAmount::new(10000)),
        };

        account.process(deposit).unwrap();

        account
    }

    #[test]
    fn deposit() {
        let mut account = Account::new();

        let deposit = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Deposit(TxAmount::new(10000)),
        };

        account.process(deposit).unwrap();

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(10000));
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::new(10000));
    }

    #[test]
    fn withdrawal() {
        let mut account = setup_account(TxAmount::new(10000));

        let withdrawal = LedgerItem {
            client_id:  1,
            tx_id:      2,
            action:     LedgerAction::Withdrawal(TxAmount::new(10000)),
        };

        account.process(withdrawal).unwrap();

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::zero());
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::zero());    
    }

    #[test]
    fn dispute() {
        let mut account = setup_account(TxAmount::new(10000));

        let dispute = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Dispute,
        };

        account.process(dispute).unwrap();

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::zero());
        assert_eq!(account.held(), TxAmount::new(10000));      
        assert_eq!(account.total(), TxAmount::new(10000));  
    }

    #[test]
    fn chargeback() {
        let mut account = setup_account(TxAmount::new(10000));

        let dispute = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Dispute,
        };

        account.process(dispute).unwrap();

        let chargeback = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Chargeback,
        };

        account.process(chargeback).unwrap();

        assert_eq!(account.is_locked(), true);
        assert_eq!(account.is_active(), false);
        
        assert_eq!(account.available(), TxAmount::zero());
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::zero());
    }

    #[test]
    fn resolve() {
        let mut account = setup_account(TxAmount::new(10000));

        let dispute = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Dispute,
        };

        account.process(dispute).unwrap();

        let resolve = LedgerItem {
            client_id:  1,
            tx_id:      1,
            action:     LedgerAction::Resolve,
        };

        account.process(resolve).unwrap();

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(10000));
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::new(10000));
    }

    #[test]
    fn overdraw() {
        let mut account = setup_account(TxAmount::new(10000));

        let withdrawal = LedgerItem {
            client_id:  1,
            tx_id:      2,
            action:     LedgerAction::Withdrawal(TxAmount::new(10001)),
        };

        assert_eq!(Err(ProcessorError::InsufficientFunds(1, 2)), account.process(withdrawal));

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(10000));
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::new(10000));
    }

    #[test]
    fn small_scenario() {
        const ITEMS: [LedgerItem; 4] = [
            LedgerItem {
                client_id:  1,
                tx_id:      1,
                action:     LedgerAction::Deposit(TxAmount::new(100)),
            },
            LedgerItem {
                client_id:  1,
                tx_id:      2,
                action:     LedgerAction::Deposit(TxAmount::new(1000)),
            },
            LedgerItem {
                client_id:  1,
                tx_id:      3,
                action:     LedgerAction::Withdrawal(TxAmount::new(100)),
            },
            LedgerItem {
                client_id:  1,
                tx_id:      4,
                action:     LedgerAction::Withdrawal(TxAmount::new(10)),
            }
        ];

        let mut account = Account::new();

        for item in ITEMS {
            account.process(item).unwrap();
        }

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(990));
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::new(990));

        account.process(LedgerItem {
            client_id:  1,
            tx_id:      2,
            action:     LedgerAction::Dispute,
        }).unwrap();

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(-10));
        assert_eq!(account.held(), TxAmount::new(1000));
        assert_eq!(account.total(), TxAmount::new(990));

        assert_eq!(Err(ProcessorError::InsufficientFunds(1, 5)), account.process(LedgerItem {
            client_id:  1,
            tx_id:      5,
            action:     LedgerAction::Withdrawal(TxAmount::new(1)),
        }));

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(-10));
        assert_eq!(account.held(), TxAmount::new(1000));
        assert_eq!(account.total(), TxAmount::new(990));

        assert_eq!(Ok(()), account.process(LedgerItem {
            client_id:  1,
            tx_id:      6,
            action:     LedgerAction::Deposit(TxAmount::new(1000)),
        }));

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(990));
        assert_eq!(account.held(), TxAmount::new(1000));
        assert_eq!(account.total(), TxAmount::new(1990));

        assert_eq!(Ok(()), account.process(LedgerItem {
            client_id:  1,
            tx_id:      7,
            action:     LedgerAction::Withdrawal(TxAmount::new(990)),
        }));

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::zero());
        assert_eq!(account.held(), TxAmount::new(1000));
        assert_eq!(account.total(), TxAmount::new(1000));

        assert_eq!(Ok(()), account.process(LedgerItem {
            client_id:  1,
            tx_id:      2,
            action:     LedgerAction::Resolve,
        }));

        assert_eq!(account.is_locked(), false);
        assert_eq!(account.is_active(), true);
        
        assert_eq!(account.available(), TxAmount::new(1000));
        assert_eq!(account.held(), TxAmount::zero());
        assert_eq!(account.total(), TxAmount::new(1000));
    }

}