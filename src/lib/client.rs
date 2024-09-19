#![allow(dead_code)]
use super::types::*;

use serde_json::Value as JsonValue;
use tracing::debug;

#[derive(Debug)]
pub enum DuneError {
    RequestError,
    ParseError,
    EncodingError,
    QueryNotFinished,
    QueryStatusError(ExecutionStatus),
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
        let client = reqwest::Client::new();
        let request_builder = client
            .post(format!(
                "https://api.dune.com/api/v1/query/{}/execute",
                query_id
            ))
            .header("X-Dune-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&ExecuteQueryParams {
                performance,
                params,
            });

        // Build the request to inspect the body
        let request = request_builder
            .try_clone()
            .expect("Failed to clone request")
            .build()
            .expect("Failed to build request");

        // Log the request body
        if let Some(body) = request.body() {
            if let Ok(body_str) = String::from_utf8(body.as_bytes().unwrap().to_vec()) {
                debug!("Request body: {}", body_str);
            }
        }

        let response = match request_builder.send().await {
            Ok(res) => {
                debug!("Response: {:#?}", res);
                res
            }
            Err(_) => return Err(DuneError::RequestError),
        };

        response
            .json::<ExecuteQueryResponse>()
            .await
            .map_err(|_| DuneError::ParseError)
    }

    pub async fn get_execution_status(
        &self,
        execution_id: &str,
    ) -> Result<ExecutionStatusResponse, DuneError> {
        let response = match reqwest::Client::new()
            .get(format!(
                "https://api.dune.com/api/v1/execution/{}/status",
                execution_id
            ))
            .header("X-Dune-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .send()
            .await
        {
            Ok(res) => res,
            Err(_) => return Err(DuneError::RequestError),
        };

        // debug!("debug: {}", response.text().await.unwrap());
        // panic!("Test panic");
        response
            .json::<ExecutionStatusResponse>()
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
            Ok(res) => {
                debug!("{:#?}", res);
                res
            }
            Err(_) => return Err(DuneError::RequestError),
        };

        let response = match response.json::<QueryResultsResponse>().await {
            Ok(res) => {
                debug!("\n\n{:#?}", res);
                res
            }
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
            debug!("\n\nnext_offset: {:?}", next_offset);
            while next_offset.is_some() {
                debug!("{:?} records processed...", params.get_offset());
                params.update_offset(next_offset.unwrap());
                params_encoded = match params.url_encode() {
                    Ok(str) => str,
                    Err(_) => return Err(DuneError::ParseError),
                };

                debug!("params_encoded (updated): {:?}", params_encoded);
                let response = match reqwest::Client::new()
                    .get(format!(
                        "https://api.dune.com/api/{}?{}",
                        &url_path, &params_encoded
                    ))
                    .header("X-Dune-API-Key", &self.api_key)
                    .send()
                    .await
                {
                    Ok(res) => {
                        debug!("{:#?}", res);
                        res
                    }
                    Err(_) => return Err(DuneError::RequestError),
                };

                let response = match response.json::<QueryResultsResponse>().await {
                    Ok(res) => {
                        debug!("{:#?}", res);
                        res
                    }
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

    pub async fn execute_query_and_get_results_when_ready(
        &self,
        query_id: u64,
        performance: EngineSize,
        params: Option<JsonValue>,
        poll_interval: Option<u64>,
        peak: bool,
    ) -> Result<QueryResult, DuneError> {
        match self.execute_query(query_id, performance, params).await {
            Ok(res) => {
                debug!("Query execution successfully submitted: {:?}", res);
                let mut has_finished = false;
                let execution_id = res.execution_id;
                while !has_finished {
                    debug!(
                        "Query execution not finished yet. Waiting {} seconds...",
                        poll_interval.unwrap_or(60)
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(
                        poll_interval.unwrap_or(60),
                    ))
                    .await;
                    match self.get_execution_status(&execution_id).await {
                        Ok(res) => match res.status {
                            ExecutionStatus::QueryStateExecuting => {}
                            ExecutionStatus::QueryStatePending => {}
                            ExecutionStatus::QueryStateCompleted => {
                                debug!("Query execution finished!");
                                has_finished = true;
                            }
                            _ => return Err(DuneError::QueryStatusError(res.status)),
                        },
                        Err(e) => {
                            debug!("Error when fetching the query results: {:?}", e);
                            return Err(e);
                        }
                    };
                }

                self.get_query_results(&execution_id, peak).await
            }
            Err(e) => {
                debug!("Error when executing the query: {:?}", e);
                return Err(e);
            }
        }
    }

    pub async fn get_query_results_when_ready(
        &self,
        execution_id: &str,
        poll_interval: Option<u64>,
        peak: bool,
    ) -> Result<QueryResult, DuneError> {
        let mut has_finished = false;
        while !has_finished {
            debug!(
                "Query execution not finished yet. Waiting {} seconds...",
                poll_interval.unwrap_or(60)
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(
                poll_interval.unwrap_or(60),
            ))
            .await;
            match self.get_execution_status(&execution_id).await {
                Ok(res) => match res.status {
                    ExecutionStatus::QueryStateExecuting => {}
                    ExecutionStatus::QueryStatePending => {}
                    ExecutionStatus::QueryStateCompleted => {
                        debug!("Query execution finished!");
                        has_finished = true;
                    }
                    _ => return Err(DuneError::QueryStatusError(res.status)),
                },
                Err(e) => {
                    debug!("Error when fetching the query results: {:?}", e);
                    return Err(e);
                }
            };
        }

        self.get_query_results(&execution_id, peak).await
    }
}
