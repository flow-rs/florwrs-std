use std::{any::Any, rc::Rc};

use flowrs::{
    connection::{Output, RuntimeConnectable},
    node::{ChangeObserver, UpdateError, InitError, ShutdownError, ReadyError, Node},
};
use flowrs_derive::Connectable;


#[derive(Connectable)]
pub struct ValueNode<I>
where
    I: Clone,
{
    name: String,
    value: I,
    
    #[output]
    pub output: Output<I>,
}

impl<I> ValueNode<I>
where
    I: Clone,
{
    pub fn new(name: &str, change_observer: &ChangeObserver, value: I) -> Self {
        Self {
            name: name.into(),
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
        println!("{:?} VALUE", std::thread::current().id());
        self.output.clone().send(self.value.clone());
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}
