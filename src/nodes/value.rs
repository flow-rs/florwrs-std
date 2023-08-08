use flowrs::{
    connection::{Output},
    node::{ChangeObserver, UpdateError, InitError, ShutdownError, ReadyError, Node},
};
use flowrs_derive::RuntimeConnectable;


#[derive(RuntimeConnectable)]
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
    I: Clone + Send + 'static,
{

    fn on_init(&self) -> Result<(), InitError>{ 
        Ok(())
    }

    fn on_ready(&self) -> Result<(), ReadyError>{
        //println!("{:?} VALUE", std::thread::current().id());
        self.output.clone().send(self.value.clone());
        Ok(())
    }

    fn on_shutdown(&self)  -> Result<(), ShutdownError> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn update(&self) -> Result<(), UpdateError> {
        Ok(())
    }
}
