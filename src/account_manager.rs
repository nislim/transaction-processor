use std::{collections::{BTreeMap, btree_map::Iter}, convert::TryFrom};

use tokio::{sync::mpsc::{Sender, channel}, task::JoinHandle};

use crate::{ClientID, LedgerItem, account::Account, error::ProcessorError, TxAmount};

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

#[derive(Debug)]
enum AccountManagerMessage {
    Process(LedgerItem),
    Dump(Sender<(ClientID, TxAmount, TxAmount, TxAmount, bool)>),
    Stop,
}

pub struct AccountManagerTask {
    handle: JoinHandle<()>,
    sender: Sender<AccountManagerMessage>,
}

impl AccountManagerTask {
    pub fn spawn() -> AccountManagerTask {
        let (sender, mut receiver) = channel(128);

        let handle = tokio::spawn(async move {
            let mut manager = AccountManager::new();
            while let Some(msg) = receiver.recv().await {
                match msg {
                    AccountManagerMessage::Process(item) => {
                        if let Err(e) = manager.process(item) {
                            eprintln!("{:?}", e);
                        }
                    },
                    AccountManagerMessage::Dump(sender) => {
                        for (client_id, account) in manager.iter() {
                            sender.send((*client_id, account.available(), account.held(), account.total(), account.is_locked())).await.unwrap()
                        }
                    }
                    AccountManagerMessage::Stop => {
                        break;
                    }
                }
            }
        });

        AccountManagerTask {
            handle,
            sender,
        }
    }

    pub async fn process(&self, item: LedgerItem) {
        self.sender.send(AccountManagerMessage::Process(item)).await.unwrap();
    }

    pub async fn dump(&self, sender: Sender<(ClientID, TxAmount, TxAmount, TxAmount, bool)>) {
        self.sender.send(AccountManagerMessage::Dump(sender)).await.unwrap()
    }

    pub async fn stop(&self) {
        self.sender.send(AccountManagerMessage::Stop).await.unwrap();
    }

    pub async fn join(self) {
        self.handle.await.unwrap()
    }
}

pub struct AccountManagerLoadbalancer {
    tasks: Vec<AccountManagerTask>,

    mask: ClientID,
}

impl AccountManagerLoadbalancer {

    pub fn spawn(count: usize) -> Self {
        assert!(count.count_ones() == 1, "Number of tasks need to be a power of 2");

        let tasks = (0..count).map(|_| AccountManagerTask::spawn()).collect();
        let mask = ClientID::try_from(count - 1).unwrap();

        AccountManagerLoadbalancer {
            tasks,
            mask,
        }
    }

    pub async fn process(&self, item: LedgerItem) {
        let index = item.client_id & self.mask;

        self.tasks[index as usize].process(item).await;
    }

    pub async fn dump(&self, sender: Sender<(ClientID, TxAmount, TxAmount, TxAmount, bool)>) {
        for task in self.tasks.iter() {
            task.dump(sender.clone()).await;
        }
    }

    pub async fn stop(&self) {
        for task in self.tasks.iter() {
            task.stop().await;
        }
    }

    pub async fn join(self) {
        for task in self.tasks {
            task.join().await;
        }
    }

}