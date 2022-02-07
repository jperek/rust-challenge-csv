use std::error::Error;
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
use client::{Client, ClientEntry};

type ClientId = u16;
type TransactionId = u32;

enum Transaction {
    Deposit(ClientId, TransactionId, Amount),
    Withdrawal(ClientId, TransactionId, Amount),
    Dispute(ClientId, TransactionId),
    Resolve(ClientId, TransactionId),
    Chargeback(ClientId, TransactionId),
}

#[derive(Debug, Deserialize)]
struct Record {
    r#type: String,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}

fn read_input_csv(path: &Path) -> Result<(), Box<dyn Error>> {
    let f = File::open(path)?;
    let reader = BufReader::new(f);
    let mut rdr = ReaderBuilder::new().trim(Trim::All).from_reader(reader);
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Record = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    let path = std::env::args().nth(1).expect("no path given");
    let path = PathBuf::from(path);
    if let Err(err) = read_input_csv(&path) {
        println!("error reading input csv file: {}", err);
        process::exit(1);
    }
}
