use std::fmt::Debug;

use flowrs_derive::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{Node, UpdateError, InitError, ShutdownError, ReadyError, ChangeObserver},
};

#[derive(RuntimeConnectable)]
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
        if let Ok(input) = self.input.next_elem() {
            println!("{:?} {:?} DEBUG", std::thread::current().id(),input);
            #[cfg(target_arch = "wasm32")]
            log(format!("{:?} {:?} DEBUG", std::thread::current().id(),input).as_str());

            match self.output.clone().send(input) {
                Ok(_) => (),
                Err(_) => return Err(UpdateError::ConnectError { node: self.name.clone(), message: "Failed to send".into() }),
            };
        }
        Ok(())
    }
}
