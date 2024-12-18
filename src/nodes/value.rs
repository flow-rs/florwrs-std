use std::fmt;
use std::str::FromStr;

use flowrs::comm::communication::NodeCommunicator;
//use flowrs::RuntimeConnectable;
use flowrs::{
    connection::Output,
    node::{Node, ReadyError, UpdateError},
};

//#[derive(RuntimeConnectable)]
pub struct ValueNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
{
    value: I,

    //#[output]
    pub output: Output<I>,
}

impl<I> ValueNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
    I: Send + 'static,
{
    // pub fn new(value: I, change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         value,
    //         output: Output::new(change_observer),
    //     }
    // }
    pub fn new(value: I, output_communicator: NodeCommunicator<I>) -> Self {
        Self {
            value,
            output: Output::new(output_communicator),
        }
    }
}

impl<I> Node for ValueNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
    I: Send + 'static,
{
    fn on_ready(&mut self) -> Result<(), ReadyError> {
        self.output
            .send(self.value.clone())
            .map_err(|e| ReadyError::Other(e.into()))?;
        Ok(())
    }

    fn get_input_count(&self) -> u128 {
        0
    }

    fn get_output_count(&self) -> u128 {
        1
    }
}

//#[derive(RuntimeConnectable)]
pub struct ValueUpdateNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
    I: Send + 'static,
{
    value: I,

    //#[output]
    pub output: Output<I>,
}

impl<I> ValueUpdateNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
    I: Send + 'static,
{
    // pub fn new(value: I, change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         value,
    //         output: Output::new(change_observer),
    //     }
    // }
    pub fn new(value: I, output_communicator: NodeCommunicator<I>) -> Self {
        Self {
            value,
            output: Output::new(output_communicator),
        }
    }
}

impl<I> Node for ValueUpdateNode<I>
where
    I: Clone,
    I: fmt::Debug,
    I: FromStr,
    I: Send + 'static,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        self.output.send(self.value.clone())?;
        Ok(())
    }

    fn get_input_count(&self) -> u128 {
        0
    }

    fn get_output_count(&self) -> u128 {
        1
    }
}
