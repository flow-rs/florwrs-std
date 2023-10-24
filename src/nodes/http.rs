use flowrs::node::SendError;
use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

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
pub struct HttpNode<I>
where
    I: Clone,
{
    #[input]
    pub data_input: Input<I>,

    #[input]
    pub config_input: Input<I>,

    #[output]
    pub output: Output<String>,
}

impl<I> HttpNode<I>
where
    I: Clone,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            data_input: Input::new(),
            config_input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for HttpNode<I>
where
    I: Clone + Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        let Ok(input) = self.data_input.next() else {
            return Err(UpdateError::Other(anyhow::Error::msg(
                "No valid data input.",
            )));
        };

        let client = Client::new();

        let response = client
            .get("https://catfact.ninja/fact")
            //.header(AUTHORIZATION, "Bearer [AUTH_TOKEN]")
            //.header(CONTENT_TYPE, "application/json")
            //.header(ACCEPT, "application/json")
            .send();

        match response {
            Ok(response) => {
                let body = response.text().unwrap();
                let _ = self.output.clone().send(body.clone());
                Ok(())
            }
            Err(e) => Err(UpdateError::Other(anyhow::Error::msg(e.to_string()))),
        }
    }
}
