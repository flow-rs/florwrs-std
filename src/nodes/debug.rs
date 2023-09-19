use std::fmt::Debug;

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{Node, UpdateError, ChangeObserver},
};
use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct DebugNode<I>
where
    I: Clone,
{
    #[input]
    pub input: Input<I>,
    #[output]
    pub output: Output<I>,
}

impl<I> DebugNode<I>
where
    I: Clone,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + Send,
{

    fn on_update(&mut self) -> Result<(), UpdateError> {
        //println!("{:?} DEBUG BEFORE ", std::thread::current().id());
        if let Ok(input) = self.input.next() {
            println!("{:?} {:?} DEBUG", std::thread::current().id(),input);
            #[cfg(target_arch = "wasm32")]
            crate::log(format!("{:?} {:?} DEBUG", std::thread::current().id(),input).as_str());
            
            // Send fails if the output is not connected. We ignore that in this case.
            let _ = self.output.send(input).map_err(|e| UpdateError::ConnectError { message: format!("Failed to send. Reason: {}", e).into() });
        }
        Ok(())
    }
}
