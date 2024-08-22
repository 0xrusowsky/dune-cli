use super::types::*;
use serde_json::Value as JsonValue;

#[derive(Debug)]
pub enum DuneError {
    RequestError,
    ParseError,
    EncodingError,
    QueryNotFinished,
}

pub struct DuneClient {
    api_key: String,
}

impl DuneClient {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn execute_query(
        &self,
        query_id: u64,
        performance: EngineSize,
        params: Option<JsonValue>,
    ) -> Result<ExecuteQueryResponse, DuneError> {
        let params = match serde_urlencoded::to_string(ExecuteQueryParams {
            performance,
            params,
        }) {
            Ok(str) => str,
            Err(_) => return Err(DuneError::EncodingError),
        };
        let response = match reqwest::Client::new()
            .post(format!(
                "https://api.dune.com/api/v1/query/{}/execute?{}",
                query_id, params
            ))
            .header("X-Dune-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(DuneError::RequestError),
        };

        response
            .json::<ExecuteQueryResponse>()
            .await
            .map_err(|_| DuneError::ParseError)
    }

    pub async fn get_query_results(&self, id: &str, peak: bool) -> Result<QueryResult, DuneError> {
        let mut rows: Vec<JsonValue> = Vec::new();
        let limit = if peak { 10 } else { 1000 };
        let (url_path, mut params) = match id.parse::<u64>() {
            // if the id is a u64, it must be a query_id
            Ok(query_id) => (
                format!("v1/query/{}/results", query_id),
                ResultsParams::Query(QueryResultsParams {
                    ignore_max_datapoints_per_request: false,
                    query_id,
                    offset: 0,
                    limit,
                    columns: None,
                }),
            ),
            // otherwise, it is an execution_id
            Err(_) => (
                format!("v1/execution/{}/results", id),
                ResultsParams::Execution(ExecutionResultsParams {
                    ignore_max_datapoints_per_request: false,
                    execution_id: id,
                    offset: 0,
                    limit,
                    columns: None,
                }),
            ),
        };
        let mut params_encoded = match params.url_encode() {
            Ok(str) => str,
            Err(_) => return Err(DuneError::EncodingError),
        };

        let response = match reqwest::Client::new()
            .get(format!(
                "https://api.dune.com/api/{}?{}",
                &url_path, &params_encoded
            ))
            .header("X-Dune-API-Key", &self.api_key)
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(DuneError::RequestError),
        };

        println!("\n{:?}", &response);
        let response = match response.json::<QueryResultsResponse>().await {
            Ok(res) => res,
            Err(_) => {
                return Err(DuneError::ParseError);
            }
        };

        if !response.is_execution_finished {
            return Err(DuneError::QueryNotFinished);
        }

        let metadata = response.result.metadata;
        rows.extend(response.result.rows);

        if !peak {
            let mut next_offset = response.next_offset;
            while next_offset.is_some() {
                println!("{:?} records processed...", params.get_offset());
                params.update_offset(next_offset.unwrap());
                params_encoded = match serde_urlencoded::to_string(&params) {
                    Ok(str) => str,
                    Err(_) => return Err(DuneError::ParseError),
                };

                let response = match reqwest::Client::new()
                    .get(format!(
                        "https://api.dune.com/api/{}?{}",
                        &url_path, &params_encoded
                    ))
                    .header("X-Dune-API-Key", &self.api_key)
                    .send()
                    .await
                {
                    Ok(res) => res,
                    Err(_) => return Err(DuneError::RequestError),
                };

                let response = match response.json::<QueryResultsResponse>().await {
                    Ok(res) => res,
                    Err(_) => {
                        return Err(DuneError::ParseError);
                    }
                };

                rows.extend(response.result.rows);
                next_offset = response.next_offset;
            }
        }

        Ok(QueryResult { metadata, rows })
    }
}
