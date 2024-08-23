mod lib;
mod utils;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use lib::{client::DuneClient, types::EngineSize};
use serde_json::Value as JsonValue;

/// Small CLI tool for executing commands of the Dune API Client.
#[derive(Parser, Debug)]
#[command(about = "Small CLI tool for executing commands of the Dune API Client.")]
struct Cli {
    /// The API key for authenticating with the Dune API.
    /// Can be provided via the env variable `DUNE_API_KEY`.
    #[clap(short = 'k', long, env = "DUNE_API_KEY")]
    api_key: Option<String>,

    /// The subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for interacting with the Dune API.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a new query with the Dune API.
    Execute {
        /// The unique identifier of the query to execute.
        #[clap(long)]
        id: u64,

        /// (Optional) Engine size to use for the query execution.
        /// Can be either "medium" or "large". Defaults to "medium".
        #[clap(long)]
        engine_size: Option<String>,

        /// (Optional) Query parameters in JSON format.
        #[clap(long)]
        params: Option<JsonValue>,
    },

    /// Retrieve results for a previously executed query.
    GetResults {
        /// The unique identifier of the execution for which to retrieve results.
        /// If a query ID is provided, the results its latest execution will be returned.
        /// If an execution ID is provided, the results for that specific execution will be returned.
        #[clap(long)]
        id: String,

        /// (Optional) Whether to retrieve all rows, or only the first 10 records.
        #[clap(short, long)]
        peak: Option<bool>,

        /// (Optional) Path where the resulting CSV file should be saved.
        #[clap(long)]
        path_csv: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli = Cli::parse();

    // ensure API key is set
    let api_key = cli
        .api_key
        .unwrap_or_else(|| std::env::var("DUNE_API_KEY").expect("DUNE_API_KEY must be set"));

    match cli.command {
        Commands::Execute {
            id,
            engine_size,
            params,
        } => {
            let performance = match engine_size {
                Some(size) => match size.to_lowercase().as_str() {
                    "medium" => EngineSize::Medium,
                    "m" => EngineSize::Medium,
                    "large" => EngineSize::Large,
                    "l" => EngineSize::Large,
                    _ => {
                        println!("Invalid performance level. Use 'medium' or 'large'.");
                        return;
                    }
                },
                None => EngineSize::Medium,
            };
            let client = DuneClient::new(api_key);
            match client.execute_query(id, performance, params).await {
                Ok(res) => println!("Response: {:?}", res),
                Err(e) => {
                    println!("Error: {:?}", e);
                    return;
                }
            };
        }
        Commands::GetResults { id, peak, path_csv } => {
            let client = DuneClient::new(api_key);
            let res = match client.get_query_results(&id, peak.unwrap_or(false)).await {
                Ok(res) => res,
                Err(e) => {
                    println!("Error: {:?}", e);
                    return;
                }
            };
            if path_csv.is_some() {
                match utils::save_json_as_csv(
                    res.rows,
                    match path_csv.unwrap().as_str() {
                        "true" => "output.csv",
                        path => path,
                    },
                )
                .await
                {
                    Ok(_) => println!("Results saved to CSV"),
                    Err(e) => println!("Error saving results to CSV file: {:?}", e),
                };
            }
        }
    }
}
