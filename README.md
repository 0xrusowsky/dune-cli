# Minimal Dune API CLI

## Overview

This CLI tool is a simple, and minimal, command-line interface for executing commands with the Dune API Client.
It allows users to run queries and retrieve results efficiently, making it easier to interact with the Dune Analytics platform.

## Features

-  Execute queries using the Dune API.
-  Retrieve results for previously executed queries.
-  Support for query parameters in JSON format.
-  Ability to save results as CSV files.

## Requirements

-  Rust
-  Dune API Key (can be set as an env variable)

## Installation

To get started with the Dune API CLI Tool, clone the repository and build the project:

```bash
git clone git@github.com:0xrusowsky/dune-cli.git
cd dune-cli
cargo build
```

## Usage

The CLI tool can be run using the following command structure:

```bash
cargo run <command> [options]
```

### Commands

#### 1. Execute a Query

Execute a new query with the Dune API.

```bash
cargo run execute --query-id <QUERY_ID> [--engine-size <ENGINE_SIZE>] [--params <PARAMS>]
```

-  `--id`: The unique identifier of the query to execute (required).
-  `--engine-size`: (Optional) The engine size to use for the query execution. Can be either `medium` or `large`. Defaults to `medium`.
-  `--params`: (Optional) Query parameters in JSON format.

**Example:**

```bash
cargo run execute --query-id 3998990 --params '{"min_lp_value_usd": 1000000000}'
```

#### 2. Get Query Results

Retrieve results for a previously executed query.

```bash
cargo run get-results --id <ID> [--peak <true|false>] [--path-csv <PATH>]
```

-  `--id`: The unique identifier of the execution for which to retrieve results (required).
-  `--peak`: (Optional) Whether to retrieve all rows or only the first 10 records.
-  `--path-csv`: (Optional) Path where the resulting CSV file should be saved.

**Example:**

```bash
cargo run get-results --id 3998990 --peak true --path-csv outputs/test.csv
```

## Environment Variables

You can set the Dune API key as an environment variable:

```bash
export DUNE_API_KEY=YOUR_API_KEY
```

or use a `.env` file in the root directory of the project with the following content:

```.env
DUNE_API_KEY=YOUR_API_KEY
```

Alternatively, you can provide it directly using the `-k` or `--api-key` option when running the CLI tool.

## Contribution

Contributions are welcome! Please feel free to submit a pull request or open an issue if you have suggestions or encounter any problems.

## License

This project is licensed under the MIT License. See the LICENSE file for more details.
