use std::collections::{BTreeMap, btree_map::Iter};

use crate::{ClientID, LedgerItem, account::Account, error::ProcessorError};

pub struct AccountManager {
    accounts: BTreeMap<ClientID, Account>
}

impl AccountManager {
    pub fn new() -> Self {
        AccountManager {
            accounts: BTreeMap::new(),
        }
    }

    pub fn process(&mut self, item: LedgerItem) -> Result<(), ProcessorError> {
        if let Some(account) = self.accounts.get_mut(&item.client_id) {
            account.process(item)
        } else {
            let mut account = Account::new();
            let client_id = item.client_id;

            if let Err(e) = account.process(item) {
                Err(e)
            } else {
                self.accounts.insert(client_id, account);

                Ok(())
            }
        }
    }

    pub fn iter(&self) -> Iter<ClientID, Account>{
        self.accounts.iter()
    }
}