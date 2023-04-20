use std::env;
use std::fs::File;
use std::io;
use serde::{Serialize, Deserialize};
use reqwest::{self, header::AUTHORIZATION};

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionList {
    data: Vec<Datum>,
    links: TransactionListLinks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Datum {
    #[serde(rename = "type")]
    datum_type: DatumType,
    id: String,
    attributes: Attributes,
    relationships: Option<Relationships>,
    links: DatumLinks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attributes {
    status: Status,
    #[serde(rename = "rawText")]
    raw_text: Option<String>,
    description: String,
    message: String,
    #[serde(rename = "isCategorizable")]
    is_categorizable: bool,
    #[serde(rename = "holdInfo")]
    hold_info: Option<serde_json::Value>,
    #[serde(rename = "roundUp")]
    round_up: Option<serde_json::Value>,
    cashback: Option<serde_json::Value>,
    amount: Amount,
    #[serde(rename = "foreignAmount")]
    foreign_amount: Option<serde_json::Value>,
    #[serde(rename = "cardPurchaseMethod")]
    card_purchase_method: Option<serde_json::Value>,
    #[serde(rename = "settledAt")]
    settled_at: String,
    #[serde(rename = "createdAt")]
    created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Amount {
    #[serde(rename = "currencyCode")]
    currency_code: CurrencyCode,
    value: String,
    #[serde(rename = "valueInBaseUnits")]
    value_in_base_units: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatumLinks {
    #[serde(rename = "self")]
    links_self: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Relationships {
    account: Account,
    #[serde(rename = "transferAccount")]
    transfer_account: Account,
    category: Category,
    #[serde(rename = "parentCategory")]
    parent_category: ParentCategory,
    tags: Category,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    data: Option<Data>,
    links: Option<AccountLinks>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(rename = "type")]
    data_type: DataType,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountLinks {
    related: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    #[serde(skip_serializing)]
    data: Option<Vec<Option<serde_json::Value>>>,
    links: Option<DatumLinks>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentCategory {
    data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionListLinks {
    prev: Option<serde_json::Value>,
    next: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CurrencyCode {
    #[serde(rename = "AUD")]
    Aud,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "SETTLED")]
    Settled,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DatumType {
    #[serde(rename = "transactions")]
    Transactions,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    #[serde(rename = "accounts")]
    Accounts,
}

#[derive(Serialize)]
pub struct TransactionCSV {
    raw_text: Option<String>,
    description: String,
    message: String,
    value_in_base_units: i64,
}

pub async fn get_transactions() -> Result<TransactionList, String> {
    let client = reqwest::Client::new();
    let transaction_url = "https://api.up.com.au/api/v1/transactions";
    let response = client.get(transaction_url)
                        .header(AUTHORIZATION, format!("Bearer {}", env::var("UP_API_TOKEN").unwrap()))
                        .query(&[("page[size]", "50")])
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
