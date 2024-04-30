//! # OpenAPI Mock Server
//!
//! `openapi-mocker` is a simple mock server for OpenAPI 3.0 specs.
//! It can be used to quickly create a mock server for an OpenAPI spec.
//!
//! The server will respond with example responses defined in the spec.
//! If no example is defined, it will respond with an empty JSON object.
//! The server will respond with a 200 status code by default, but you can
//! specify a different status code in the URL.
//!
//! ## Usage
//! ```sh
//! openapi-mocker <spec> [port]
//! ```
//! * `<spec>` - Path to the OpenAPI spec file
//! * `[port]` - Port to bind the server to (default: 8080)
//!
//! ## Example
//! ```sh
//! openapi-mocker tests/testdata/petstore.yaml
//! ```
//! This will start a server on port 8080 with the Petstore spec.
//! You can then make requests to the server to get example responses.
//! For example, to get a list of pets:
//! ```sh
//! curl http://localhost:8080/200/pets
//! ```
//! This will return a list of pets from the example response in the spec.
//! You can also specify a different status code in the URL:
//! ```sh
//! curl http://localhost:8080/404/pets
//! ```
//! This will return a 404 status code with the example response for a 404 error.
//!
use clap::Parser;
use std::path::PathBuf;
pub mod server;
pub mod spec;

#[derive(Parser)]
#[clap(version = "0.1.0", author = "Thiago Pacheco")]
pub struct Args {
    #[clap(index = 1)]
    pub spec: PathBuf,
    #[clap(short, long, default_value = "8080")]
    pub port: Option<u16>,
}
