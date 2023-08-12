use std::ops::Div;

use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use flowrs_derive::RuntimeConnectable;

enum DivNodeState<I1, I2> {
    I1(I1),
    I2(I2),
    None,
}

#[derive(RuntimeConnectable)]
pub struct DivNode<I1, I2, O>
where
    I1: Clone,
    I2: Clone,
{
    
    state: DivNodeState<I1, I2>,

    #[input]
    pub input_1: Input<I1>,
    #[input]
    pub input_2: Input<I2>,
    #[output]
    pub output_1: Output<O>,
}

impl<I1, I2, O> DivNode<I1, I2, O>
where
    I1: Clone + Div<I2, Output = O> + Send,
    I2: Clone + Send,
    O: Clone + Send,
{
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
           
            state: DivNodeState::None,
            input_1: Input::new(),
            input_2: Input::new(),
            output_1: Output::new(change_observer),
        }
    }

    fn handle_1(&mut self, v: I1) -> Result<(), UpdateError> {
        match &self.state {
            DivNodeState::I1(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Division should happen pairwise.".into(),
                })
            }
            DivNodeState::I2(i) => {
                let out = v / i.clone();
                self.state = DivNodeState::None;
                self.output_1.clone().send(out)?;
            }
            DivNodeState::None => self.state = DivNodeState::I1(v),
        }
        Ok(())
    }

    fn handle_2(&mut self, v: I2) -> Result<(), UpdateError> {
        match &self.state {
            DivNodeState::I2(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Division should happen pairwise.".into(),
                })
            }
            DivNodeState::I1(i) => {
                let out = i.clone() / v;
                self.state = DivNodeState::None;
                self.output_1.clone().send(out)?;
            }
            DivNodeState::None => self.state = DivNodeState::I2(v),
        }
        Ok(())
    }
}

impl<I1, I2, O> Node for DivNode<I1, I2, O>
where
    I1: Div<I2, Output = O> + Clone + Send,
    I2: Clone + Send,
    O: Clone + Send,
{

    // To be replaced by macro
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(i1) = self.input_1.next() {
            self.handle_1(i1)?;
        }
        if let Ok(i2) = self.input_2.next() {
            self.handle_2(i2)?;
        }
        Ok(())
    }
}

#[test]
fn should_div_132() -> Result<(), UpdateError> {
    let change_observer = ChangeObserver::new();

    let mut add: DivNode<f32, f32, f32> = DivNode::new(Some(&change_observer));
    let mock_output = flowrs::connection::Edge::new();
    flowrs::connection::connect(add.output_1.clone(), mock_output.clone());
    let _ = add.input_1.send(15.0);
    let _ = add.input_2.send(2.0);
    let _ = add.on_update();
    let _ = add.on_update();

    let expected = 7.5;
    let actual = mock_output.next()?;
    Ok(assert!(expected == actual))
}
