use std::collections::HashMap;
use std::str::FromStr;

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use reqwest::Method;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(RuntimeConnectable, Deserialize, Serialize)]

/*
Data Input:
{
    "url": "",
    "method": "",
    "headers": {
        "": "",
        "": ""
    },
    "body": ""
}

Config Input
{
    "timeout": 10000
}
*/
pub struct HttpNode {
    #[input]
    pub data_input: Input<serde_json::Value>,

    #[input]
    pub config_input: Input<serde_json::Value>,

    #[output]
    pub output: Output<String>,
}

impl HttpNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            data_input: Input::new(),
            config_input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

fn extract_key_value_pairs(json_value: &Value) -> HeaderMap{
    //let mut headers = HashMap::new();
    let mut header_map = reqwest::header::HeaderMap::new();


    if let Value::Object(raw_header_map) = json_value {
        for (key, value) in raw_header_map {
            //if let Value::String(header_value) = value {
            //    headers.insert(key.to_string(), header_value.to_string());
            //}
            let header_name = HeaderName::from_str(&key.to_lowercase());
            if header_name.is_err() {
                continue;
            }
        
            let value_str = &value.as_str().unwrap_or("").to_lowercase();
            let header_value = HeaderValue::from_str(&value_str).unwrap_or(HeaderValue::from_static(""));
            header_map.insert(header_name.unwrap(), header_value);
            }
    }

    // headers
    header_map
}

impl Node for HttpNode {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        let input = self.data_input.next()?;

        let mut client_builder = Client::builder();

        let method_str = match &input["method"] {
            Value::String(m) => Ok(m.clone()),
            _ => Err(UpdateError::Other(anyhow::Error::msg(
                "No REST method given. Key \"method\" with a valid REST verb as value is required. Currently supported: GET.",
            ))),
        }?;

        let url = match &input["url"] {
            Value::String(u) => Ok(u.clone()),
            _ => Err(UpdateError::Other(anyhow::Error::msg(
                "No URL given.",
            ))),
        }?;

        let headers = extract_key_value_pairs(&input.get("headers").unwrap_or(&Value::Null));
        
        client_builder = client_builder.default_headers(headers);

        let body_value = input.get("body").unwrap_or(&Value::Null);
        let body = serde_json::to_string(&body_value).unwrap_or(String::from(""));

        let reqwest_err_map = |e: reqwest::Error| UpdateError::Other(anyhow::Error::msg(e.to_string()));
        
        let built_client = client_builder.build().map_err(reqwest_err_map)?;

        let method = match method_str.to_lowercase().as_str() {
            "get" => Ok(Method::GET),
            "post" => Ok(Method::POST),
            "put" => Ok(Method::PUT), // not tested
            "delete" => Ok(Method::DELETE), // not tested
            other => Err(UpdateError::Other(anyhow::Error::msg(
                format!("HTTP request method \"{other}\" not implemented or not valid."),
            ))),
        }?;

        let response = built_client.request(method, url).body(body).send();

        match response {
            Ok(response) => {
                let body = response.text().unwrap();
                self.output.send(body)?; // TODO: return body + headers
                Ok(())
            }
            Err(e) => Err(UpdateError::Other(anyhow::Error::msg(e.to_string()))),
        }
    }
}
