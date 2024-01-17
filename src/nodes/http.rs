use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use log::{warn, debug};

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// Own implementation of a HttpMethod enum in order to achieve abstraction of the used HTTP request library.
#[derive(Clone, Copy, Deserialize, Debug)]
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    HEAD,
    OPTIONS,
    PATCH
}

/// Object to specify a HTTP request and is supplied via [`HttpNode::data_input`].
#[derive(Clone, Deserialize, Debug)]
pub struct RequestInput {
    pub url: String,
    pub method: HTTPMethod,
    /// Invalid HTTP headers will be ignored and not sent with the request. Also all header names will be converted to lower case.
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Configures [`HttpNode`] for all future requests if supplied once via [`HttpNode::config_input`]. If never supplied, the default configuration is used (see source of [`HttpNode::new()`]).
#[derive(Clone, Deserialize, Debug)]
pub struct ConfigInput {
    /// Timeout for all HTTP Requests.
    pub timeout: Option<Duration>,
    /// Controls if the HTTP client should accept invalid certificates. **Dangerous if set to `true`! Use only in development!**
    pub accept_invalid_certs: Option<bool>,
}

/// The output object of a executed HTTP request with [`HttpNode::on_update()`].
#[derive(Clone, Debug)]
pub struct ResponseOutput {
    pub body: String,
    pub headers: HashMap<String, String>,
    pub response_code: u16,
    pub content_length: u64,
}

/// Node which sends HTTP Requests.
#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct HttpNode {
    /// Input to execute a single HTTP request via [`Self::on_update()`].
    #[input]
    pub data_input: Input<RequestInput>,

    /// Configures [`HttpNode`] for all future requests. If never changed, the default configuration is used (see source of [`Self::new()`]).
    #[input]
    pub config_input: Input<ConfigInput>,

    /// The output of a executed HTTP request with [`Self::on_update()`].
    #[output]
    pub output: Output<ResponseOutput>,

    timeout: Duration,
    accept_invalid_certs: bool,
}

impl HttpNode {
    /// Creates a new HttpNode instance with the default configuration.
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            data_input: Input::new(),
            config_input: Input::new(),
            output: Output::new(change_observer),
            // Default timeout configuration.
            timeout: Duration::new(30, 0),
            // By default invalid certificates are not accepted.
            accept_invalid_certs: false,
        }
    }

    /// Shows if the HTTP client accepts invalid certificates. **Dangerous! Should only be set to `true` in development!** This parameter is set via [`Self::config_input`].
    pub fn accept_invalid_certs(&self) -> bool {
        self.accept_invalid_certs
    }

    /// Timeout for all HTTP Requests. This parameter is set via [`Self::config_input`].
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

fn extract_key_value_pairs(raw_header_hashmap: &HashMap<String, String>) -> HeaderMap {
    let mut header_map = HeaderMap::new();

    for (key, value) in raw_header_hashmap {
        let header_name = HeaderName::from_str(&key.to_lowercase());
        if header_name.is_err() {
            warn!("Invalid header name (and its header value) is discarded and is not sent with request: {}", key);
            continue;
        }

        let value_str = &value.to_lowercase();
        let header_value = HeaderValue::from_str(&value_str);
        if header_value.is_err() {
            warn!("Invalid header value is discarded and complete header key-value pair is not sent with request: {}: {}", header_name.unwrap(), value);
            continue;
        }
        header_map.insert(header_name.unwrap(), header_value.unwrap());
    }

    header_map
}

fn convert_header_map(header_map: &HeaderMap) -> HashMap<String, String> {
    let mut hash_map = HashMap::new();

    for (key, value) in header_map {
        let key_str = key.as_str().to_string();
        hash_map.insert(key_str, value.to_str().unwrap().to_string());
    }
    debug!("Conversion result of received HeaderMap into HashMap: {:?}", hash_map);
    hash_map
}

impl Node for HttpNode {
    /// Executes a HTTP request with the values specified in the current [`RequestInput`] object (supplied via [`Self::data_input`]) and the last supplied [`ConfigInput`] object (supplied via [`Self::config_input`]). If no [`ConfigInput`] object was ever supplied, the default values are used (see source of [`Self::new()`]).
    ///
    /// # Example
    ///
    /// ```
    /// use flowrs::{
    ///     connection::{connect, Edge},
    ///     node::{ChangeObserver, Node},
    /// };
    /// use flowrs_std::http::{ConfigInput, HTTPMethod, HttpNode, RequestInput};
    /// use std::{collections::HashMap, time::Duration};
    ///
    /// let mut server = mockito::Server::new();
    /// let url = server.url();
    /// let method = HTTPMethod::GET;
    /// let path = "/hello&question=how%20are%20you";
    /// let expected_response_body = "Hello World!";
    ///
    /// // Create a mock
    /// let mock = server
    ///     .mock("GET", path)
    ///     .with_status(200)
    ///     .with_body(expected_response_body)
    ///     .create();
    ///
    /// let change_observer: ChangeObserver = ChangeObserver::new();
    /// let data_input = RequestInput {
    ///     url: url.to_string() + path,
    ///     method,
    ///     headers: HashMap::new(),
    ///     body: None,
    /// };
    ///
    /// let new_timeout = 5000;
    /// let config_input = ConfigInput {
    ///     timeout: Some(Duration::from_millis(new_timeout)),
    ///     accept_invalid_certs: Some(false),
    /// };
    ///
    /// let mut http_node: HttpNode = HttpNode::new(Some(&change_observer));
    /// let mock_output = Edge::new();
    /// connect(http_node.output.clone(), mock_output.clone());
    /// let _ = http_node.config_input.send(config_input);
    /// let _ = http_node.data_input.send(data_input);
    /// http_node.on_update().unwrap();

    /// mock.assert(); // checks if the mock server has been called
    /// let returned_body = mock_output.next().unwrap();
    /// assert!(returned_body.body == expected_response_body);
    /// assert!(http_node.timeout() == Duration::from_millis(new_timeout));
    /// ```
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(config) = self.config_input.next() {
            if let Some(timeout) = config.timeout {
                self.timeout = timeout;
                debug!("New timeout for HTTP requests: {:?}", self.timeout)
            }
            if let Some(accept_invalid_certs) = config.accept_invalid_certs {
                self.accept_invalid_certs = accept_invalid_certs;
                debug!("New accept_invalid_certs for HTTP requests: {:?}", self.accept_invalid_certs)
            }
        }

        let Ok(input) = self.data_input.next() else {
            return Ok(());
        };
        debug!("Received RequestInput object: {:?}", input);

        let mut client_builder = Client::builder();

        let headers = extract_key_value_pairs(&input.headers);
        debug!("Headers parsed into HeaderMap: {:?}", headers);

        let body = input.body.unwrap_or_default();

        let reqwest_err_map =
            |e: reqwest::Error| UpdateError::Other(anyhow::Error::msg(e.to_string()));

        debug!("Current configuration: timeout: {:?}, accept_invalid_certs: {:?}", self.timeout, self.accept_invalid_certs);
        client_builder = client_builder
            .default_headers(headers)
            .timeout(self.timeout)
            .danger_accept_invalid_certs(self.accept_invalid_certs);
        let built_client = client_builder.build().map_err(reqwest_err_map)?;

        let method = match &input.method {
            HTTPMethod::GET => Method::GET,
            HTTPMethod::POST => Method::POST,
            HTTPMethod::PUT => Method::PUT,
            HTTPMethod::OPTIONS => Method::OPTIONS,
            HTTPMethod::DELETE => Method::DELETE,
            HTTPMethod::HEAD => Method::HEAD,
            HTTPMethod::PATCH => Method::PATCH,
        };

        let response = built_client.request(method, input.url).body(body).send();

        match response {
            Ok(response) => {
                debug!("Received response: {:?}", response);
                debug!("Received response headers: {:?}", response.headers());
                let headers = response.headers().clone();
                let response_code = response.status().as_u16();
                let content_length = response.content_length().unwrap_or_default();
                let body = response.text().unwrap();
                debug!("Received response body: {:?}", body);
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
