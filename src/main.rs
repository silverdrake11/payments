use std::env;
use std::collections::HashMap;

use csv;
use serde::Deserialize;


#[derive(Debug, Deserialize)]
struct Record {
  #[serde(rename = "type")] // To avoid using keyword
  tx_type: String,
  client: u16, // u16 specified in PDF
  tx: u32, // u32 specified in PDF
  amount: Option<f64>, // Sometimes amount is not specified
}


#[derive(Debug, PartialEq)]
struct Client {
  available: f64,
  held: f64,
  locked: bool,
}

impl Client {

  fn new() -> Client {
    Client {
      available: 0.0,
      held: 0.0,
      locked: false,
    }
  }

  fn total(&self) -> f64 {
    return self.available + self.held;
  }

}

fn print_clients(clients: &HashMap<u16,Client>) {
  println!("client,available,held,total,locked");
  for (client_id, client) in clients {
    println!("{},{:.4},{:.4},{:.4},{}",
      client_id,
      client.available,
      client.held,
      client.total(),
      client.locked);
  }
}

fn process_txs(filepath: &str) -> HashMap<u16,Client> {

  let mut reader = csv::ReaderBuilder::new()
      .trim(csv::Trim::All) // Trim whitespace from hearders and fields
      .from_path(filepath)
      .expect("The path specified is not valid!");

  let mut clients: HashMap<u16,Client> = HashMap::new(); // client_id -> Client
  let mut transactions: HashMap<u32,f64> = HashMap::new(); // Transaction amounts

  for row in reader.deserialize() {

    let record: Record = row.expect("Line in CSV not properly formatted!");

    // If a transaction amount is specified, save it
    let tx_id = record.tx;
    if let Some(value) = record.amount {
      transactions.insert(tx_id, value);
    }

    // Get client or insert new if client ID doesn't exist
    let mut client = clients.entry(record.client)
        .or_insert(Client::new());

    if client.locked { // We don't won't to modify if account is frozen
      continue;
    }

    let tx_type = record.tx_type.as_str();

    match tx_type { // Process the transaction here

      "deposit" => client.available += record.amount
          .expect("Amount missing in deposit!"),

      "withdrawal" => {
        let amount = record.amount
            .expect("Amount missing in withdrawal!");
        if client.available >= amount {
          client.available -= amount;
        }
      },

      _ => { // Not a deposit or withdrawal

        if !transactions.contains_key(&tx_id) {
          continue; // Ignore if tx id does not exist
        }
        let tx_amount = transactions[&tx_id];

        match tx_type {
          "dispute" => {
            client.available -= tx_amount;
            client.held += tx_amount;
          },

          "resolve" => {
            client.available += tx_amount;
            client.held -= tx_amount;
          },

          "chargeback" => {
            client.held -= tx_amount;
            client.locked = true;
          },
          _ => panic!("Invalid command '{:?}'", tx_type),}

      },
    }
  }

  return clients;
}

fn main() {

  // Read path from command line
  let args: Vec<String> = env::args().collect();
  let filepath = args.get(1)
      .expect("The csv filepath must be the first argument!");

  let clients = process_txs(filepath);
  print_clients(&clients);
}


#[test]
fn sample_test() { // Sample test from PDF
  let clients = process_txs("sample.csv");
  assert_eq!(clients.len(), 2);
  assert_eq!(clients[&1], 
    Client{available:1.5, held:0.0, locked:false});
  assert_eq!(clients[&2], 
    Client{available:2.0, held:0.0, locked:false});
}

#[test]
fn test1_chargeback() { // Makes account cannot be modified here
  let clients = process_txs("test1.csv");
  assert_eq!(clients[&1], 
    Client{available:0.0, held:0.0, locked:true});
}

#[test]
fn test2_resolve() { // After resolve things showed work smoothly
  let clients = process_txs("test2.csv");
  assert_eq!(clients[&1], 
    Client{available:3.0, held:0.0, locked:false});
}

#[test]
fn test3_dispute_withdraw() { // We should not be able to withdraw after dispute
  let clients = process_txs("test3.csv");
  assert_eq!(clients[&1], 
    Client{available:0.0, held:1.0, locked:false});
}