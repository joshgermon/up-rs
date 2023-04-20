mod transactions;

use dotenv::dotenv;
use transactions::{get_transactions, parse_transactions};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    // default command is build
    #[clap(name = "transactions")]
    Transactions,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    match args.command {
        Command::Transactions => {
            let transactions = get_transactions().await;
            parse_transactions(transactions.unwrap());
        }
    }
}

