use std::{fmt::Debug};

use flowrs_derive::Connectable;
use flowrs::{
    connection::{Input, Output},
    node::{Node, UpdateError, ChangeObserver},
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
    pub fn new(name: &str, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            name: name.into(),
            input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + Send,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn on_update(&mut self) -> Result<(), UpdateError> {
        //println!("{:?} DEBUG BEFORE ", std::thread::current().id());
        if let Ok(input) = self.input.next_elem() {
            //println!("{:?} {:?} DEBUG", std::thread::current().id(),input);

            self.output.clone().send(input).unwrap();
        }
        Ok(())
    }
}
