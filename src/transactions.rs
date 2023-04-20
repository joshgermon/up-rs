use std::env;
use std::fs::File;
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
    raw_text: Option<String>,
    description: String,
    message: Option<String>,
    value_in_base_units: i64,
}

pub async fn get_transactions() -> Result<TransactionList, String> {
    let client = reqwest::Client::new();
    let response = client.get(TRANSACTIONS_ENDPOINT)
                        .header(AUTHORIZATION, format!("Bearer {}", env::var("UP_API_TOKEN").unwrap()))
                        .query(&[("page[size]", "100")])
                        .send().await.unwrap();

    match response.status() {
        reqwest::StatusCode::OK => {
            let transaction_list = response.json::<TransactionList>().await.map_err(|e| format!("JSON deserialization error: {}", e))?;
            Ok(transaction_list)
        }
        reqwest::StatusCode::UNAUTHORIZED => {
            Err(String::from("Authentication error"))
        }
        _ => {
            Err(format!("Unknown error: {}", response.status()))
        }
    }
}


pub fn parse_transactions(transactions: TransactionList) -> Result<(), String> {
    let list = transactions.data;
    let file = File::create("test.csv").unwrap();
    let mut wtr = csv::WriterBuilder::new().has_headers(true).from_writer(file);
    let income: Vec<TransactionCSV> = list.into_iter().filter(|t| {
            t.attributes.amount.value_in_base_units > 0
        }
    )
    .map(|t| { TransactionCSV {
        raw_text: t.attributes.raw_text,
        description: t.attributes.description,
        message: t.attributes.message,
        value_in_base_units: t.attributes.amount.value_in_base_units
    }})
    .collect();

    for txn in income {
        wtr.serialize(txn).unwrap();
    }

    wtr.flush().unwrap();
    Ok(())
}

// pub fn parse_transactions(transactions: TransactionList) -> Result<(), String> {
//     let list = transactions.data;
//     let income: Vec<Datum> = list.into_iter().filter(|t| {
//             t.attributes.amount.value_in_base_units > 0
//         }
//     ).collect();
//     let names_only: Vec<String>= income.into_iter().filter_map(|t| t.attributes.raw_text ).collect();
//     println!("{:?}", names_only);
//     Ok(())
// }
