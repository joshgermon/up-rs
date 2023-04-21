mod transactions;

use dotenv::dotenv;
use transactions::{get_transactions, parse_transactions, write_to_csv};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
pub enum Command {
    #[clap(name = "transactions")]
    Transactions(TransactionsArgs),
}

#[derive(Debug, Parser)]
pub struct TransactionsArgs {
    #[arg(short, long)]
    size: i8,
}


#[tokio::main]
async fn main() {
    dotenv().ok();
    let args = Args::parse();
    match args.command {
        Command::Transactions(transactions_args) => {
            let transactions = get_transactions(transactions_args.size).await;
            let csv_data = parse_transactions(transactions.unwrap());
            write_to_csv(csv_data.unwrap(), "transaction.csv");
        }
    }
}

