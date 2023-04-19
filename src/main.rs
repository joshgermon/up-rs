use std::env;
use dotenv::dotenv;
use reqwest::{self, header::AUTHORIZATION};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TransactionList {
    data: Vec<Datum>,
    links: TransactionListLinks,
}

#[derive(Debug, Deserialize)]
pub struct Datum {
    #[serde(rename = "type")]
    datum_type: String,
    id: String,
    attributes: Attributes,
    relationships: Relationships,
    links: DatumLinks,
}

#[derive(Debug, Deserialize)]
pub struct Attributes {
    status: Option<String>,
    #[serde(rename = "rawText")]
    raw_text: Option<String>,
    description: Option<String>,
    message: Option<String>,
    #[serde(rename = "isCategorizable")]
    is_categorizable: Option<bool>,
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
    settled_at: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Amount {
    #[serde(rename = "currencyCode")]
    currency_code: Option<String>,
    value: Option<String>,
    #[serde(rename = "valueInBaseUnits")]
    value_in_base_units: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DatumLinks {
    #[serde(rename = "self")]
    links_self: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Relationships {
    account: Option<Account>,
    #[serde(rename = "transferAccount")]
    transfer_account: Option<ParentCategory>,
    category: Option<Category>,
    #[serde(rename = "parentCategory")]
    parent_category: Option<ParentCategory>,
    tags: Option<Category>,
}

#[derive(Debug, Deserialize)]
pub struct Account {
    data: Option<Data>,
    links: Option<AccountLinks>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    #[serde(rename = "type")]
    data_type: Option<String>,
    id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccountLinks {
    related: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Category {
    data: Option<Vec<Option<serde_json::Value>>>,
    links: Option<DatumLinks>,
}

#[derive(Debug, Deserialize)]
pub struct ParentCategory {
    data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TransactionListLinks {
    prev: Option<serde_json::Value>,
    next: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let transactions = get_transactions().await;
    parse_transactions(transactions.unwrap());
}

async fn get_transactions() -> Result<TransactionList, String> {
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

fn parse_transactions(transactions: TransactionList) -> Result<(), String> {
    let list = transactions.data;
    let income: Vec<Datum> = list.into_iter().filter(|t| {
            if let Some(value) = t.attributes.amount.value_in_base_units {
                return value > 0;
            }
            false
        }
    ).collect();
    let names_only: Vec<String>= income.into_iter().filter_map(|t| t.attributes.raw_text ).collect();
    println!("{:?}", names_only);
    Ok(())
}
