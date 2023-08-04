use std::{any::Any, fmt::Debug, rc::Rc};

use flowrs_derive::Connectable;
use flowrs::{
    connection::{Input, Output, RuntimeConnectable},
    node::{Node, UpdateError, InitError, ShutdownError, ReadyError, ChangeObserver},
};

#[derive(Connectable)]
pub struct DebugNode<I>
where
    I: Clone,
{
    name: String,

    #[input]
    pub input: Input<I>,
    #[output]
    pub output: Output<I>,
}

impl<I> DebugNode<I>
where
    I: Clone,
{
    pub fn new(name: &str, change_observer: &ChangeObserver) -> Self {
        Self {
            name: name.into(),
            input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + Send + 'static,
{
    fn on_init(&self) -> Result<(), InitError>{ 
        Ok(())
    }

    fn on_ready(&self)   -> Result<(), ReadyError>{
        Ok(())
    }

    fn on_shutdown(&self)  -> Result<(), ShutdownError> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn update(&self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next_elem() {
            println!("{:?} {:?} DEBUG", std::thread::current().id(),input);

            self.output.clone().send(input).unwrap();
        }
        Ok(())
    }
}
