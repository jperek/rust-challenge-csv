use std::collections::HashMap;
use std::error::Error;
use std::io::Write;
use std::io::{stdout, BufWriter};
use std::process;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use csv::{ReaderBuilder, Trim};
use serde::Deserialize;

mod amount;
use amount::Amount;

mod client;
use client::{Client, ClientTransaction};

type ClientId = u16;
type TransactionId = u32;

enum Transaction {
    Deposit(ClientId, TransactionId, Amount),
    Withdrawal(ClientId, TransactionId, Amount),
    Dispute(ClientId, TransactionId),
    Resolve(ClientId, TransactionId),
    Chargeback(ClientId, TransactionId),
}

impl Transaction {
    pub fn from_record(record: Record) -> Self {
        match record.r#type.as_str() {
            "deposit" => Transaction::Deposit(record.client, record.tx, record.amount.unwrap()),
            "withdrawal" => {
                Transaction::Withdrawal(record.client, record.tx, record.amount.unwrap())
            }
            "dispute" => Transaction::Dispute(record.client, record.tx),
            "resolve" => Transaction::Resolve(record.client, record.tx),
            "chargeback" => Transaction::Chargeback(record.client, record.tx),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Record {
    r#type: String,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}

struct Database {
    clients: HashMap<ClientId, Client>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_transaction(&mut self, tx: Transaction) {
        let (client_id, client_tx) = match tx {
            Transaction::Deposit(client_id, tx_id, amount) => {
                (client_id, ClientTransaction::deposit(tx_id, amount))
            }
            Transaction::Withdrawal(client_id, tx_id, amount) => {
                (client_id, ClientTransaction::withdrawal(tx_id, amount))
            }
            Transaction::Dispute(client_id, tx_id) => {
                (client_id, ClientTransaction::dispute(tx_id))
            }
            Transaction::Resolve(client_id, tx_id) => {
                (client_id, ClientTransaction::resolve(tx_id))
            }
            Transaction::Chargeback(client_id, tx_id) => {
                (client_id, ClientTransaction::chargeback(tx_id))
            }
        };

        if let Some(client) = self.clients.get_mut(&client_id) {
            client.add_transaction(client_tx)
        } else {
            let mut client = Client::new(client_id);
            client.add_transaction(client_tx);
            self.clients.insert(client_id, client);
        }
    }

    pub fn write_all(&self, writer: &mut dyn Write) -> Result<(), Box<dyn Error>> {
        writeln!(writer, "client,available,held,total,locked")?;
        for client in self.clients.values() {
            let entry = client.get_entry();
            writeln!(writer, "{}", entry)?;
        }
        Ok(())
    }
}

fn read_input_csv(path: &Path, database: &mut Database) -> Result<(), Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(reader);
    for result in rdr.deserialize() {
        let record: Record = result?;
        database.add_transaction(Transaction::from_record(record));
    }
    Ok(())
}

fn write_output(database: &Database) -> Result<(), Box<dyn Error>> {
    let mut writer = BufWriter::new(stdout());
    database.write_all(&mut writer)?;
    Ok(())
}

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let path = PathBuf::from(path);

    let mut database = Database::new();

    if let Err(err) = read_input_csv(&path, &mut database) {
        println!("error reading input csv file: {}", err);
        process::exit(1);
    }

    if let Err(err) = write_output(&database) {
        println!("error writing output: {}", err);
        process::exit(1);
    }
}
