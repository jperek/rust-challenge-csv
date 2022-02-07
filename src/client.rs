use std::{collections::HashMap, fmt};

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
                if self.transactions.iter().find(|other_tx| {
                    other_tx.id == transaction.id
                        && (matches!(other_tx.tx_type, ClientTransactionType::Deposit)
                            || matches!(other_tx.tx_type, ClientTransactionType::Withdrawal))
                }).is_some() {
                    self.transactions.push(transaction);
                }
            }
            ClientTransactionType::Resolve | ClientTransactionType::Chargeback => {
                if self.transactions.iter().find(|other_tx| {
                    other_tx.id == transaction.id
                        && matches!(other_tx.tx_type, ClientTransactionType::Dispute)
                }).is_some() {
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
                    if let Some(tx_found) = self.transactions.iter().find(|other_tx| {
                        other_tx.id == tx.id
                            && (matches!(other_tx.tx_type, ClientTransactionType::Deposit)
                                || matches!(other_tx.tx_type, ClientTransactionType::Withdrawal))
                    }) {
                        let mut amount = tx_found.amount.unwrap();
                        if matches!(tx_found.tx_type, ClientTransactionType::Withdrawal) {
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

        ClientEntry::new(self.id, available, held, locked)
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
    id: TransactionId,
    tx_type: ClientTransactionType,
    amount: Option<Amount>,
}

impl ClientTransaction {
    pub fn deposit(id: TransactionId, amount: Amount) -> Self {
        Self {
            id,
            tx_type: ClientTransactionType::Deposit,
            amount: Some(amount),
        }
    }

    pub fn withdrawal(id: TransactionId, amount: Amount) -> Self {
        Self {
            id,
            tx_type: ClientTransactionType::Withdrawal,
            amount: Some(amount),
        }
    }

    pub fn dispute(id: TransactionId) -> Self {
        Self {
            id,
            tx_type: ClientTransactionType::Dispute,
            amount: None,
        }
    }

    pub fn resolve(id: TransactionId) -> Self {
        Self {
            id,
            tx_type: ClientTransactionType::Resolve,
            amount: None,
        }
    }

    pub fn chargeback(id: TransactionId) -> Self {
        Self {
            id,
            tx_type: ClientTransactionType::Chargeback,
            amount: None,
        }
    }
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

    #[test]
    fn deposit_resolve() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(100000)));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,false");

        client.add_transaction(ClientTransaction::dispute(1));
        assert_eq!(format!("{}", client.get_entry()), "1,0,10,10,false");

        client.add_transaction(ClientTransaction::resolve(1));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,false");
    }

    #[test]
    fn deposit_chargeback() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(100000)));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,false");

        client.add_transaction(ClientTransaction::dispute(1));
        assert_eq!(format!("{}", client.get_entry()), "1,0,10,10,false");

        client.add_transaction(ClientTransaction::chargeback(1));
        assert_eq!(format!("{}", client.get_entry()), "1,0,0,0,true");
    }

    #[test]
    fn withdrawal_resolve() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(100000)));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,false");

        client.add_transaction(ClientTransaction::withdrawal(2, Amount::new(20000)));
        assert_eq!(format!("{}", client.get_entry()), "1,8,0,8,false");

        client.add_transaction(ClientTransaction::dispute(2));
        assert_eq!(format!("{}", client.get_entry()), "1,10,-2,8,false");

        client.add_transaction(ClientTransaction::resolve(2));
        assert_eq!(format!("{}", client.get_entry()), "1,8,0,8,false");
    }

    #[test]
    fn withdrawal_chargeback() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(100000)));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,false");

        client.add_transaction(ClientTransaction::withdrawal(2, Amount::new(20000)));
        assert_eq!(format!("{}", client.get_entry()), "1,8,0,8,false");

        client.add_transaction(ClientTransaction::dispute(2));
        assert_eq!(format!("{}", client.get_entry()), "1,10,-2,8,false");

        client.add_transaction(ClientTransaction::chargeback(2));
        assert_eq!(format!("{}", client.get_entry()), "1,10,0,10,true");
    }

    #[test]
    fn cannot_spend_over_available_funds() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(10000)));
        assert_eq!(format!("{}", client.get_entry()), "1,1,0,1,false");

        client.add_transaction(ClientTransaction::withdrawal(2, Amount::new(20000)));
        assert_eq!(format!("{}", client.get_entry()), "1,1,0,1,false");
    }

    #[test]
    fn negative_available_funds() {
        let mut client = Client::new(1);

        client.add_transaction(ClientTransaction::deposit(1, Amount::new(10000)));
        assert_eq!(format!("{}", client.get_entry()), "1,1,0,1,false");

        client.add_transaction(ClientTransaction::withdrawal(2, Amount::new(5000)));
        assert_eq!(format!("{}", client.get_entry()), "1,0.5,0,0.5,false");

        client.add_transaction(ClientTransaction::dispute(1));
        assert_eq!(format!("{}", client.get_entry()), "1,-0.5,1,0.5,false");

        client.add_transaction(ClientTransaction::chargeback(1));
        assert_eq!(format!("{}", client.get_entry()), "1,-0.5,0,-0.5,true");
    }
}
