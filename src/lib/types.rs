#![allow(dead_code)]

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
    #[serde(rename = "state", deserialize_with = "deserialize_status")]
    pub status: ExecutionStatus,
}

// GET: QUERY EXECUTION STATE
#[derive(Debug, Deserialize)]
pub struct ExecutionStatusResponse {
    pub execution_id: String,
    pub query_id: u64,
    pub is_execution_finished: bool,
    pub result_metadata: Option<StatusResultMetadata>,
    #[serde(rename = "state", deserialize_with = "deserialize_status")]
    pub status: ExecutionStatus,
}

#[derive(Debug, Deserialize)]
pub struct StatusResultMetadata {
    pub column_names: Vec<String>,
    pub column_types: Vec<String>,
    pub datapoint_count: u64,
    pub total_row_count: u64,
}

#[derive(Debug, Deserialize, PartialEq)]
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

// Custom deserializer for ExecutionStatus
fn deserialize_status<'de, D>(deserializer: D) -> Result<ExecutionStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "QUERY_STATE_PENDING" => Ok(ExecutionStatus::QueryStatePending),
        "QUERY_STATE_EXECUTING" => Ok(ExecutionStatus::QueryStateExecuting),
        "QUERY_STATE_FAILED" => Ok(ExecutionStatus::QueryStateFailed),
        "QUERY_STATE_COMPLETED" => Ok(ExecutionStatus::QueryStateCompleted),
        "QUERY_STATE_CANCELLED" => Ok(ExecutionStatus::QueryStateCancelled),
        "QUERY_STATE_EXPIRED" => Ok(ExecutionStatus::QueryStateExpired),
        "QUERY_STATE_COMPLETED_PARTIAL" => Ok(ExecutionStatus::QueryStateCompletedPartial),
        _ => Err(serde::de::Error::custom(format!("Invalid variant: {}", s))),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finished_execution_status_response() {
        let response: &str = r#"
            {
                "execution_id": "01J5ZMD33P6J413G1KQM6QTE4S",
                "query_id": 4011227,
                "is_execution_finished": true,
                "state": "QUERY_STATE_COMPLETED",
                "submitted_at": "2024-08-23T12:46:55.606607Z",
                "expires_at": "2024-11-21T13:05:39.370484Z",
                "execution_started_at": "2024-08-23T12:46:57.221499084Z",
                "execution_ended_at": "2024-08-23T13:05:39.370482549Z",
                "result_metadata": {
                    "column_names": ["address", "balance", "balance_usd"],
                    "column_types": ["varbinary", "double", "double"],
                    "row_count": 1068677,
                    "result_set_bytes": 61983266,
                    "total_row_count": 1068677,
                    "total_result_set_bytes": 61983266,
                    "datapoint_count": 3206031,
                    "pending_time_millis": 1614,
                    "execution_time_millis": 1122148
                }
            }
            "#;

        let response: ExecutionStatusResponse = serde_json::from_str(response).unwrap();

        // Assert the values are correctly parsed
        assert_eq!(response.execution_id, "01J5ZMD33P6J413G1KQM6QTE4S");
        assert_eq!(response.query_id, 4011227);
        assert!(response.is_execution_finished);
        assert_eq!(response.status, ExecutionStatus::QueryStateCompleted);

        let metadata = response.result_metadata.unwrap();
        assert_eq!(
            metadata.column_names,
            vec!["address", "balance", "balance_usd"]
        );
        assert_eq!(metadata.column_types, vec!["varbinary", "double", "double"]);
        assert_eq!(metadata.total_row_count, 1068677);
        assert_eq!(metadata.datapoint_count, 3206031);
    }

    #[test]
    fn test_in_progress_execution_status_response() {
        let response: &str = r#"
            {
                "execution_id":"01J5ZV5R55K2MA1943RFX994B3",
                "query_id":4011227,
                "is_execution_finished":false,
                "state":"QUERY_STATE_EXECUTING",
                "submitted_at":"2024-08-23T14:45:15.045773Z",
                "execution_started_at":"2024-08-23T14:45:16.717963921Z"
            }
            "#;

        let response: ExecutionStatusResponse = serde_json::from_str(response).unwrap();

        // Assert the values are correctly parsed
        assert_eq!(response.execution_id, "01J5ZV5R55K2MA1943RFX994B3");
        assert_eq!(response.query_id, 4011227);
        assert!(!response.is_execution_finished);
        assert_eq!(response.status, ExecutionStatus::QueryStateExecuting);
    }
}
