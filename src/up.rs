use reqwest::{self, header::AUTHORIZATION};

const BASE_URL: &str = "https://api.up.com.au/api/v1";
const TRANSACTIONS_ENDPOINT: &str = format!("{}/transactions", BASE_URL);

#[derive(Serialize, Deserialize)]
pub struct UpResponse {
    pub data: Vec<TransactionResource>,
    pub links: MetaLinks
}
