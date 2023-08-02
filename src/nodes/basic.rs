use std::{any::Any, fmt::Debug, rc::Rc};

use flowrs::{
    connection::{Output, RuntimeConnectable},
    node::{State, UpdateError, Node},
};
use flowrs_derive::Connectable;

use flowrs::node::Context;

#[derive(Connectable)]
pub struct BasicNode<I>
where
    I: Clone,
{
    name: String,
    _state: State<Option<I>>,
    props: I,
    _context: State<Context>,

    #[output]
    pub output: Output<I>,
}

impl<I> BasicNode<I>
where
    I: Clone,
{
    pub fn new(name: &str, context: State<Context>, props: I) -> Self {
        Self {
            name: name.into(),
            _state: State::new(None),
            props,
            _context: context.clone(),
            output: Output::new(context.clone()),
        }
    }
}

impl<I> Node for BasicNode<I>
where
    I: Clone + Debug + Send + 'static,
{
    fn on_init(&self) {
        ()
    }

    fn on_ready(&self) {
        let elem = &self.props;
        self.output.clone().send(elem.clone()).unwrap();
    }

    fn on_shutdown(&self) {}

    fn name(&self) -> &str {
        &self.name
    }

    fn update(&self) -> Result<(), UpdateError> {
        Ok(())
    }
}
