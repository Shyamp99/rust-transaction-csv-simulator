use csv;
use serde::{Serialize, Deserialize, Deserializer};
use eyre::{Result, WrapErr};

use std::any::type_name;
use std::borrow::BorrowMut;
use std::{vec::Vec, error::Error, collections::HashMap, cell::RefCell};
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::io;
use std::env;
use std::process;

#[derive(Debug, Deserialize)]
struct Transaction{
    #[serde(rename = "type")]
    transaction_type: String,
    client: u16,
    tx: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    amount: Option<f64>,
}

fn default_dispute_status() -> Option<bool> {
    return Some(false);
}

#[derive(Debug)]
struct Client{
    id: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
    transactions: RefCell<HashMap<u32, Transaction>>,
    disputes: RefCell<HashMap<u32, f64>>
}

fn build_client(id: u16, amount: f64) -> Option<Client>{
    if amount == -1.0{
        return None;
    }
    return Some(Client{
        id: id, 
        available: amount, 
        held: 0.0,
        total: amount,
        locked: false,
        transactions: RefCell::new(HashMap::new()),
        disputes: RefCell::new(HashMap::new()),
    })
}

fn type_of<T>(_: T) -> &'static str {
    type_name::<T>()
}

fn parse_input_csv() -> Result<Vec<Transaction>> {
    // Build the CSV reader and iterate over each record.
    let mut vec:Vec<Transaction> = Vec::new();
    let path:String = env::args().nth(1).expect("no csv path given"); 
    let mut rdr = csv::Reader::from_path(path)?;
    for result in rdr.deserialize() {
        match result.ok(){
            None => continue,
            Some(transaction) => vec.push(transaction),
        }
    }
    Ok(vec)
}


fn aggregate_data(transactions: Vec<Transaction>) -> (){
    let mut clients: HashMap<u16, Client> = HashMap::new();
    for transaction in transactions{
        let client_id = transaction.client;
        let mut transaction_amount =  match transaction.amount{
            Some(amount) => amount,
            None => -1.0
        };
        if transaction.transaction_type.to_lowercase().eq("deposit"){
            match clients.entry(client_id){
                Occupied(mut client_optional) => {
                    let client = client_optional.get_mut();
                    if transaction_amount >= 0.0 && !client.locked{
                        client.available += transaction_amount;
                        client.total += transaction_amount;
                        client.transactions.borrow_mut().insert(transaction.tx, transaction);
                    }
                },
                Vacant(Client) => {
                    if transaction_amount >= 0.0{
                        if let Some(new_client) = build_client(client_id, transaction_amount){
                            clients.insert(client_id, new_client);
                            // not using optional returned from the insert because it always seemed to return a Option<None> in testing
                            clients.get(&client_id).unwrap().transactions.borrow_mut().insert(transaction.tx, transaction);
                        }
                    }
                }
            };
        }
        else if transaction.transaction_type.to_lowercase().eq("withdrawal") {
            match clients.entry(client_id){
                Occupied(mut client_optional) => {
                    let client = client_optional.get_mut();
                    if transaction_amount <= client.available && transaction_amount >= 0.0 && !client.locked{
                        transaction_amount = f64::trunc(transaction_amount * 10000.0) / 10000.0;
                        client.available -= transaction_amount;
                        client.total -= transaction_amount;
                        client.transactions.borrow_mut().insert(transaction.tx, transaction);
                    }
                },
                // ignoring withdrawals if an account doesn't already exist or hasn't been deposited in
                Vacant(Client) => {}
            };
        }
        else if transaction.transaction_type.to_lowercase().eq("dispute"){
            clients.entry(client_id).and_modify(|client| {
                // update_client_balance(client, transaction.amount, false)
                let associated_transaction = client.transactions.get_mut().get_mut(&transaction.tx);
                if !associated_transaction.is_none() && !client.locked{
                    transaction_amount = f64::trunc(associated_transaction.unwrap().amount.unwrap()* 10000.0) / 10000.0;
                    client.held += transaction_amount;
                    client.available -= transaction_amount;
                    client.disputes.borrow_mut().insert(transaction.tx, transaction_amount);
                }
            });
        }
        else if transaction.transaction_type.to_lowercase().eq("resolve"){
            clients.entry(client_id).and_modify(|client| {
                // update_client_balance(client, transaction.amount, false)
                let associated_transaction = client.transactions.get_mut().get(&transaction.tx);
                let associated_dispute = client.disputes.get_mut().get(&transaction.tx);
                if !associated_transaction.is_none() && !associated_dispute.is_none() && !client.locked{
                    transaction_amount = *associated_dispute.unwrap();
                    client.held -= transaction_amount;
                    client.available += transaction_amount;
                    let x = client.disputes.borrow_mut().remove_entry(&associated_transaction.unwrap().tx);
                }
            });
        }
        else if transaction.transaction_type.to_lowercase().eq("chargeback"){
            clients.entry(client_id).and_modify(|client| {
                let associated_transaction = client.transactions.get_mut().get(&transaction.tx);
                let associated_dispute = client.disputes.get_mut().get(&transaction.tx);
                if !associated_transaction.is_none() && !associated_dispute.is_none() && !client.locked{
                    transaction_amount = *associated_dispute.unwrap();
                    client.total -= transaction_amount;
                    client.held -= transaction_amount;
                    client.locked = true;
                    client.disputes.borrow_mut().remove_entry(&associated_transaction.unwrap().tx);
                }
            });
        }
        else{
            // continuing here to skip possible malformed datapoints
            continue;
        }
    }
output_csv(clients);
}

fn output_csv(clients: HashMap<u16, Client>) -> (){
    print!("client,available,held,total,locked\n");
    for client in clients.values(){
            print!("{:?},{:.4},{:.4},{:.4},{:?}\n", client.id, client.available, client.held, client.total, client.locked)
    }
}

fn main() {
    match parse_input_csv() {
        Ok(transactions) => aggregate_data(transactions),
        Err(result) => panic!("ERROR: failure in reading information from {:?}", env::args().nth(1).expect("")),
    }
}
