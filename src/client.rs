use std::{fmt, collections::HashMap};

use crate::{Amount, ClientId, TransactionId};

pub struct Client {
    id: ClientId,
    transactions: Vec<ClientTransaction>,
}

impl Client {
    pub fn new(id: ClientId) -> Self {
        Self {
            id,
            transactions: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, transaction: ClientTransaction) {
        match transaction.tx_type {
            ClientTransactionType::Deposit | ClientTransactionType::Withdrawal => {
                if transaction.amount.unwrap() != Amount::new(0) {
                    self.transactions.push(transaction);
                }
            }
            ClientTransactionType::Dispute => {
                if let Some(_) = self.transactions.iter().find(|other_tx| {
                    other_tx.id == transaction.id
                        && (matches!(other_tx.tx_type, ClientTransactionType::Deposit)
                            || matches!(other_tx.tx_type, ClientTransactionType::Withdrawal))
                }) {
                    self.transactions.push(transaction);
                }
            }
            ClientTransactionType::Resolve | ClientTransactionType::Chargeback => {
                if let Some(_) = self.transactions.iter().find(|other_tx| {
                    other_tx.id == transaction.id
                        && matches!(other_tx.tx_type, ClientTransactionType::Dispute)
                }) {
                    self.transactions.push(transaction);
                }
            }
        }
    }

    pub fn get_entry(&self) -> ClientEntry {
        let mut available = Amount::new(0);
        let mut held = Amount::new(0);
        let mut locked = false;

        let mut disputed: HashMap<TransactionId, Amount> = HashMap::new();

        for tx in &self.transactions {
            match tx.tx_type {
                ClientTransactionType::Deposit => {
                    let amount = tx.amount.unwrap();
                    available += amount;
                }
                ClientTransactionType::Withdrawal => {
                    let amount = tx.amount.unwrap();
                    if available >= amount && !locked {
                        available -= amount;
                    }
                }
                ClientTransactionType::Dispute => {
                    if let Some(tx) = self.transactions.iter().find(|other_tx| {
                        other_tx.id == tx.id
                            && (matches!(other_tx.tx_type, ClientTransactionType::Deposit)
                                || matches!(other_tx.tx_type, ClientTransactionType::Withdrawal))
                    }) {
                        let mut amount = tx.amount.unwrap();
                        if matches!(tx.tx_type, ClientTransactionType::Withdrawal) {
                            amount = Amount::new(0) - amount;
                        }
                        disputed.insert(tx.id, amount);
                        available -= amount;
                        held += amount;
                    }
                }
                ClientTransactionType::Resolve => {
                    if let Some(amount) = disputed.get(&tx.id) {
                        held -= *amount;
                        available += *amount;
                        disputed.remove(&tx.id);
                    }
                }
                ClientTransactionType::Chargeback => {
                    if let Some(amount) = disputed.get(&tx.id) {
                        locked = true;
                        held -= *amount;
                        disputed.remove(&tx.id);
                    }
                }
            }
        }

        ClientEntry {
            id: self.id,
            available,
            held,
            locked,
        }
    }
}

pub struct ClientEntry {
    id: ClientId,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl ClientEntry {
    pub fn new(id: ClientId, available: Amount, held: Amount, locked: bool) -> Self {
        Self {
            id,
            available,
            held,
            locked,
        }
    }
}

impl fmt::Display for ClientEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{},{},{},{},{}",
            self.id,
            self.available,
            self.held,
            self.available + self.held,
            self.locked
        )
    }
}

#[derive(Copy, Clone)]
pub enum ClientTransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

pub struct ClientTransaction {
    tx_type: ClientTransactionType,
    id: TransactionId,
    amount: Option<Amount>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formatting() {
        let id = 1;
        let available = Amount::new(12345);
        let held = Amount::new(1);
        let locked = false;
        let entry = ClientEntry::new(id, available, held, locked);
        assert_eq!(format!("{}", entry), "1,1.2345,0.0001,1.2346,false");
    }
}
