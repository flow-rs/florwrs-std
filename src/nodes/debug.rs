use std::{any::Any, fmt::Debug, rc::Rc};

use flowrs_derive::Connectable;
use serde_json::Value;

use flowrs::{
    connection::{Input, Output, RuntimeConnectable},
    node::{Node, State, UpdateError, Context},
};

#[derive(Connectable)]
pub struct DebugNode<I>
where
    I: Clone,
{
    name: String,
    _state: State<Option<I>>,
    _props: Value,
    _context: State<Context>,

    #[input]
    pub input: Input<I>,
    #[output]
    pub output: Output<I>,
}

impl<I> DebugNode<I>
where
    I: Clone,
{
    pub fn new(name: &str, context: State<Context>, props: Value) -> Self {
        Self {
            name: name.into(),
            _state: State::new(None),
            _props: props,
            _context: context.clone(),
            input: Input::new(),
            output: Output::new(context.clone()),
        }
    }
}

impl<I> Node for DebugNode<I>
where
    I: Clone + Debug + Send + 'static,
{
    fn on_init(&self) {}

    fn on_ready(&self) {}

    fn on_shutdown(&self) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn update(&self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next_elem() {
            println!("{:?}", input);
            self.output.clone().send(input).unwrap();
        }
        Ok(())
    }
}
