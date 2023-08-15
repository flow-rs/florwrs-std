use flowrs::{
    connection::Output,
    node::{ChangeObserver, ReadyError, Node},
};
use flowrs_derive::RuntimeConnectable;
use serde::{Deserialize, Serialize};


#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct ValueNode<I>
where
    I: Clone,
{
    value: I,
    
    #[output]
    pub output: Output<I>,
}

impl<I> ValueNode<I>
where
    I: Clone,
{
    pub fn new(value: I, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            value,
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for ValueNode<I>
where
    I: Clone + Send,
{
    fn on_ready(&self) -> Result<(), ReadyError>{
        self.output.clone().send(self.value.clone()).map_err(|e| ReadyError::Other(e.into()))?;   
        Ok(())
    }
}
