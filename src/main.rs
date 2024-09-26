mod lib;
mod utils;

use clap::{Parser, Subcommand};
use dotenv::dotenv;
use lib::{
    client::DuneClient,
    types::{EngineSize, QueryResultsFilter},
};
use serde_json::Value as JsonValue;
use tracing::{debug, error, info};

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

    /// Retrieve the execution status of a previously executed query.
    GetStatus {
        /// The unique identifier of the execution for which to retrieve results.
        #[clap(long)]
        id: String,
    },

    /// Retrieve metadata of a materialized view.
    GetMaterializedView {
        /// The unique identifier (name) of the materialized view for which to retrieve data.
        #[clap(long)]
        id: String,
    },

    /// Retrieve results for a previously executed query.
    GetResults {
        /// The unique identifier of the execution for which to retrieve results.
        /// If a query ID is provided, the results its latest execution will be returned.
        /// If an execution ID is provided, the results for that specific execution will be returned.
        #[clap(long)]
        id: String,

        /// (Optional) Whether to apply a filter to the results or not.
        #[clap(short, long)]
        filter: Option<String>,

        /// (Optional) Whether to retrieve all rows, or only the first 10 records.
        #[clap(short, long)]
        peak: Option<bool>,

        /// (Optional) Path where the resulting CSV file should be saved.
        #[clap(long)]
        path_csv: Option<String>,
    },

    /// Execute a new query with the Dune API and wait until the results are ready.
    ExecuteGetResults {
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
    let tracing_sub = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(tracing_sub)
        .expect("Setting tracing subscriber failed");

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
                        error!("Invalid performance level. Use 'medium' or 'large'.");
                        return;
                    }
                },
                None => EngineSize::Medium,
            };
            let client = DuneClient::new(api_key);
            match client.execute_query(id, performance, params).await {
                Ok(res) => info!("Response: {:?}", res),
                Err(e) => {
                    error!("Error: {:?}", e);
                    return;
                }
            };
        }
        Commands::GetStatus { id } => {
            let client = DuneClient::new(api_key);
            match client.get_execution_status(&id).await {
                Ok(res) => info!("Response: {:?}", res),
                Err(e) => {
                    error!("Error: {:?}", e);
                    return;
                }
            };
        }
        Commands::GetMaterializedView { id } => {
            let client = DuneClient::new(api_key);
            match client.get_materialized_view_results(&id).await {
                Ok(res) => info!("Response: {:?}", res),
                Err(e) => {
                    error!("Error: {:?}", e);
                    return;
                }
            };
        }
        Commands::GetResults {
            id,
            filter,
            peak,
            path_csv,
        } => {
            let client = DuneClient::new(api_key);
            let res = match client
                .get_query_results(
                    &id,
                    match filter {
                        Some(filter) => QueryResultsFilter::new().add_filter(filter),
                        None => QueryResultsFilter::new(),
                    },
                    peak.unwrap_or(false),
                )
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    error!("Error: {:?}", e);
                    return;
                }
            };

            // save results to CSV if path is provided
            match path_csv {
                Some(path_csv) => {
                    match utils::save_json_as_csv(
                        res.rows,
                        match path_csv.as_str() {
                            "true" => "output.csv",
                            path => path,
                        },
                    )
                    .await
                    {
                        Ok(_) => info!("Results saved to CSV"),
                        Err(e) => error!("Error saving results to CSV file: {:?}", e),
                    };
                }
                None => info!("Results: {:?}", res),
            }
        }
        Commands::ExecuteGetResults {
            id,
            engine_size,
            params,
            peak,
            path_csv,
        } => {
            let performance = match engine_size {
                Some(size) => match size.to_lowercase().as_str() {
                    "medium" => EngineSize::Medium,
                    "m" => EngineSize::Medium,
                    "large" => EngineSize::Large,
                    "l" => EngineSize::Large,
                    _ => {
                        error!("Invalid performance level. Use 'medium' or 'large'.");
                        return;
                    }
                },
                None => EngineSize::Medium,
            };
            let client = DuneClient::new(api_key);
            let res = match client
                .execute_query_and_get_results_when_ready(
                    id,
                    performance,
                    params,
                    None,
                    peak.unwrap_or(false),
                )
                .await
            {
                Ok(res) => res,
                Err(e) => {
                    error!("Error: {:?}", e);
                    return;
                }
            };

            // save results to CSV if path is provided
            match path_csv {
                Some(path_csv) => {
                    match utils::save_json_as_csv(
                        res.rows,
                        match path_csv.as_str() {
                            "true" => "output.csv",
                            path => path,
                        },
                    )
                    .await
                    {
                        Ok(_) => info!("Results saved to CSV"),
                        Err(e) => error!("Error saving results to CSV file: {:?}", e),
                    };
                }
                None => info!("Results: {:?}", res),
            }
        }
    }
}
