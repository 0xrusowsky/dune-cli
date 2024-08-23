use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value as JsonValue;

// QUERY PARAMS

#[derive(Debug, Clone)]
pub enum EngineSize {
    Large,
    Medium,
}

impl Serialize for EngineSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            EngineSize::Large => serializer.serialize_str("large"),
            EngineSize::Medium => serializer.serialize_str("medium"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Blockchain {
    Ethereum,
    Arbitrum,
    Optimism,
    Base,
    Polygon,
}

impl Serialize for Blockchain {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Blockchain::Ethereum => serializer.serialize_str("ethereum"),
            Blockchain::Arbitrum => serializer.serialize_str("arbitrum"),
            Blockchain::Optimism => serializer.serialize_str("optimism"),
            Blockchain::Base => serializer.serialize_str("base"),
            Blockchain::Polygon => serializer.serialize_str("polygon"),
        }
    }
}

// POST: EXECUTE QUERY

#[derive(Debug, Serialize)]
pub struct ExecuteQueryParams {
    pub performance: EngineSize,
    #[serde(rename = "query_parameters")]
    pub params: Option<JsonValue>,
}

#[derive(Debug, Deserialize)]
pub struct ExecuteQueryResponse {
    pub execution_id: String,
    pub state: String,
}

// GET: QUERY EXECUTION STATE
#[derive(Debug, Deserialize)]
pub struct ExecutionStatusResponse {
    pub is_execution_finished: bool,
    pub datapoint_count: u128,
    pub total_row_count: u128,
    pub status: ExecutionStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExecutionStatus {
    QueryStatePending,
    QueryStateExecuting,
    QueryStateFailed,
    QueryStateCompleted,
    QueryStateCancelled,
    QueryStateExpired,
    QueryStateCompletedPartial,
}

// GET: QUERY EXECUTION RESULTS

#[derive(Debug, Serialize)]
pub enum ResultsParams<'a> {
    Query(QueryResultsParams),
    Execution(ExecutionResultsParams<'a>),
}

impl<'a> ResultsParams<'a> {
    pub fn update_offset(&mut self, new_offset: u64) {
        match self {
            ResultsParams::Query(ref mut query_params) => {
                query_params.offset = new_offset;
            }
            ResultsParams::Execution(ref mut execution_params) => {
                execution_params.offset = new_offset;
            }
        }
    }

    pub fn get_offset(&self) -> u64 {
        match self {
            ResultsParams::Query(query_params) => query_params.offset,
            ResultsParams::Execution(execution_params) => execution_params.offset,
        }
    }

    pub fn url_encode(&self) -> Result<String, serde_urlencoded::ser::Error> {
        match self {
            ResultsParams::Query(query_params) => serde_urlencoded::to_string(query_params),
            ResultsParams::Execution(execution_params) => {
                serde_urlencoded::to_string(execution_params)
            }
        }
    }
}

// to get the results of a specific query execution
#[derive(Debug, Serialize)]
pub struct ExecutionResultsParams<'a> {
    pub columns: Option<Vec<String>>,
    pub execution_id: &'a str,
    pub offset: u64,
    pub limit: u64,
    pub ignore_max_datapoints_per_request: bool,
}

// to get the results of the latest execution of a query
#[derive(Debug, Serialize)]
pub struct QueryResultsParams {
    pub columns: Option<Vec<String>>,
    pub query_id: u64,
    pub offset: u64,
    pub limit: u64,
    pub ignore_max_datapoints_per_request: bool,
}

#[derive(Debug, Deserialize)]
pub struct QueryResultsResponse {
    pub state: String,
    pub execution_id: String,
    pub is_execution_finished: bool,
    pub next_offset: Option<u64>,
    pub query_id: u64,
    pub result: QueryResult,
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryResult {
    pub metadata: QueryResultMetadata,
    pub rows: Vec<JsonValue>,
}

#[derive(Debug, Deserialize, Default)]
pub struct QueryResultMetadata {
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
    pub datapoint_count: u128,
    pub total_row_count: u128,
    pub row_count: u128,
}
