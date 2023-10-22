use flowrs::RuntimeConnectable;
use flowrs::node::SendError;
use flowrs::{
    connection::{Input, Output},
    node::{Node, UpdateError, ChangeObserver},
};
use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct HttpNode<I>
where
    I: Clone,
{
    #[input]
    pub data_input: Input<I>,

    #[input]
    pub config_input: Input<I>,

    #[output]
    pub output: Output<I>,
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
        if let Ok(input) = self.data_input.next() {
            //let client = reqwest::Client::new();
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
                    //let _ = self.output.clone().send(body.clone());
                },
                Err(e) => {
                    //UpdateError::new(e.to_string());
                    //e.to_string()
                    //let _ = self.output.send(SendError::new(e.to_string().clone()));
                }
            };

            // Send fails if the output is not connected. We ignore that in //this case.
            //let _ = self.output.send(update_object);
        }
        Ok(())
    }
}
