use std::env;
use std::fs::File;
use log::info;
use serde::{Serialize, Deserialize};
use reqwest::{self, header::AUTHORIZATION};

const TRANSACTIONS_ENDPOINT: &str = "https://api.up.com.au/api/v1/transactions";

#[derive(Serialize, Deserialize)]
pub struct TransactionList {
    pub data: Vec<TransactionResource>,
    pub links: MetaLinks
}

#[derive(Serialize, Deserialize)]
pub struct MetaLinks {
    pub prev: Option<String>,
    pub next: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct TransactionResource {
    #[serde(rename = "type")]
    pub _type: String,
    pub id: String,
    pub attributes: TransactionAttributes,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionAttributes {
    pub status: TransactionStatusEnum,
    pub raw_text: Option<String>,
    pub description: String,
    pub message: Option<String>,
    pub is_categorizable: bool,
    pub amount: MoneyObject,
    pub settled_at: String,
    pub created_at: String
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TransactionStatusEnum {
    HELD,
    SETTLED,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldInfoObject {
    pub amount: MoneyObject,
    pub foreign_amount: Option<MoneyObject>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoneyObject {
    pub currency_code: String,
    pub value: String,
    pub value_in_base_units: i64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundUpObject {
    pub amount: MoneyObject,
    pub boost_portion: Option<MoneyObject>,
}

#[derive(Serialize, Deserialize)]
pub struct CashbackObject {
    pub description: String,
    pub amount: MoneyObject,
}

#[derive(Serialize)]
pub struct TransactionCSV {
    status: TransactionStatusEnum,
    raw_text: Option<String>,
    description: String,
    message: Option<String>,
    amount: String,
    value_in_base_units: i64,
    created_at: String
}

pub async fn get_transactions(args: TransactionsArgs) -> Result<Vec<TransactionResource>, String> {
    let client = reqwest::Client::new();
    let mut transaction_list = Vec::new();
    let mut current_link = Some(String::from(TRANSACTIONS_ENDPOINT));
    println!("Requesting transactions from Up API...");
    while let Some(endpoint) = current_link {
        let response = client.get(endpoint)
                            .header(AUTHORIZATION, format!("Bearer {}", env::var("UP_API_TOKEN").unwrap()))
                            .query(&[("page[size]", args.size.to_string())])
                            .send().await.unwrap();

        match response.status() {
            reqwest::StatusCode::OK => {
                let transactions = response.json::<TransactionList>().await.map_err(|e| format!("JSON deserialization error: {}", e))?;
                transaction_list.extend(transactions.data);
                current_link = transactions.links.next;
            }
            reqwest::StatusCode::UNAUTHORIZED => {
                return Err(String::from("Authentication error"))
            }
            _ => {
                return Err(format!("Unknown error: {}", response.status()))
            }
        }
    }
    println!("Retrieved all transactions.");
    Ok(transaction_list)
}

pub fn parse_transactions(transactions: Vec<TransactionResource>) -> Result<Vec<TransactionCSV>, String> {
    /* Filter out expenses */
    let income: Vec<TransactionCSV> = transactions.into_iter().filter(|t| {
            t.attributes.amount.value_in_base_units > 0
        }
    )
    .map(|t| { TransactionCSV {
        status: t.attributes.status,
        raw_text: t.attributes.raw_text,
        description: t.attributes.description,
        message: t.attributes.message,
        amount: t.attributes.amount.value,
        value_in_base_units: t.attributes.amount.value_in_base_units,
        created_at: t.attributes.created_at
    }})
    .collect();
    Ok(income)
}

pub fn write_to_csv(rows: Vec<TransactionCSV>, file_path: &str) -> Result<(), std::io::Error>{
    println!("Creating csv file at {}", file_path);
    let file = File::create(file_path).unwrap();
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(file);
    info!("Writing records into CSV...");
    for row in rows {
        wtr.serialize(row)?;
    }
    wtr.flush()?;
    println!("Completed writing all records.");
    Ok(())
}
