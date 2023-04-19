mod transactions;

use dotenv::dotenv;
use transactions::{get_transactions, parse_transactions};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    name: String,

    /// Number of times to greet
    #[arg(short, long, default_value_t = 1)]
    count: u8,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    let transactions = get_transactions().await;
    parse_transactions(transactions.unwrap());
}

