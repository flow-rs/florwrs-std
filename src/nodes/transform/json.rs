use flowrs::{node::{Node, UpdateError, ChangeObserver}, connection::{Input, Output}};

use flowrs_derive::Connectable;

use serde::{Deserialize, Serialize};

#[derive(Connectable)]
pub struct ToJsonStringNode<T> {
    pub output: Output<String>,
    pub input: Input<T>,
}

impl<T> ToJsonStringNode<T> 
where T: Serialize {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for ToJsonStringNode<T>
where T: Serialize + Send {

    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next_elem() {
            
            let res = serde_json::to_string(&input);
            match res {
                Ok(json_str) => {
                    _ = self.output.send(json_str);
                    return Ok(())
                },
                Err(err) => {
                    return Err(UpdateError::Other(err.into()));
                }
            }
        }
        Ok(())
    }
}

#[derive(Connectable)]
pub struct FromJsonStringNode<T> {
    pub output: Output<T>,
    pub input: Input<String>,
}

impl<T> FromJsonStringNode<T>
    where T: for<'a> Deserialize<'a> + Send {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for FromJsonStringNode<T> 
    where T: for<'a> Deserialize<'a> + Send {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next_elem() {
            
            let res = serde_json::from_str(input.as_str());
            match res {
                Ok(obj) => {
                    _ = self.output.send(obj);
                    return Ok(())
                },
                Err(err) => {
                    return Err(UpdateError::Other(err.into()));
                }
            }
        }
        Ok(())
    }
}