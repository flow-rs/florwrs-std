use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

#[derive(Clone)]
pub struct RequestInput {
    pub url: String,
    pub method: HTTPMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

pub struct ConfigInput {
    pub timeout: Option<Duration>,
    pub accept_invalid_certs: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct ResponseOutput {
    pub body: String,
    pub headers: HashMap<String, String>,
    pub response_code: u16,
    pub content_length: u64,
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct HttpNode {
    #[input]
    pub data_input: Input<RequestInput>, // struct

    #[input]
    pub config_input: Input<ConfigInput>, // struct

    #[output]
    pub output: Output<ResponseOutput>,

    pub timeout: Duration,
    pub accept_invalid_certs: bool,
}

impl HttpNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            data_input: Input::new(),
            config_input: Input::new(),
            output: Output::new(change_observer),
            timeout: Duration::new(30, 0),
            accept_invalid_certs: false,
        }
    }
}

fn extract_key_value_pairs(raw_header_hashmap: &HashMap<String, String>) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (key, value) in raw_header_hashmap {
        let header_name = HeaderName::from_str(&key.to_lowercase());
        if header_name.is_err() {
            continue;
        }

        let value_str = &value.to_lowercase();
        let header_value =
            HeaderValue::from_str(&value_str).unwrap_or_else(|_| HeaderValue::from_static(""));
        header_map.insert(header_name.unwrap(), header_value);
    }

    header_map
}

fn convert_header_map(header_map: &HeaderMap) -> HashMap<String, String> {
    let mut hash_map = HashMap::new();

    for (key, value) in header_map {
        let key_str = key.as_str().to_string();
        hash_map.insert(key_str, value.to_str().unwrap().to_string());
    }

    hash_map
}

impl Node for HttpNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(config) = self.config_input.next() {
            if let Some(timeout) = config.timeout {
                self.timeout = timeout;
            }
            if let Some(accept_invalid_certs) = config.accept_invalid_certs {
                self.accept_invalid_certs = accept_invalid_certs;
            }
        }

        let Ok(input) = self.data_input.next() else {
            return Ok(());
        };

        let mut client_builder = Client::builder();

        let headers = extract_key_value_pairs(&input.headers);

        let body = input.body.unwrap_or_default();
        //let body = serde_json::to_string(&body_value).unwrap_or_default();

        let reqwest_err_map =
            |e: reqwest::Error| UpdateError::Other(anyhow::Error::msg(e.to_string()));

        client_builder = client_builder
            .default_headers(headers)
            .timeout(self.timeout)
            .danger_accept_invalid_certs(self.accept_invalid_certs);
        let built_client = client_builder.build().map_err(reqwest_err_map)?;

        let method = match &input.method {
            HTTPMethod::GET => Method::GET,
            HTTPMethod::POST => Method::POST,
            HTTPMethod::PUT => Method::PUT,       // not tested
            HTTPMethod::DELETE => Method::DELETE, // not tested
        };

        let response = built_client.request(method, input.url).body(body).send();

        match response {
            Ok(response) => {
                let headers = response.headers().clone();
                let response_code = response.status().as_u16();
                let content_length = response.content_length().unwrap_or_default();
                let body = response.text().unwrap();
                let output = ResponseOutput {
                    body,
                    headers: convert_header_map(&headers),
                    response_code,
                    content_length,
                };
                self.output.send(output.clone())?;
                Ok(())
            }
            Err(e) => Err(UpdateError::Other(anyhow::Error::msg(e.to_string()))),
        }
    }
}
