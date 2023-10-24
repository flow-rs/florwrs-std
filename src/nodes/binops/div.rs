use super::binops::BinOpState;
use crate::handle_sequentially;
use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
};
use serde::{Deserialize, Serialize};
use std::ops::Div;

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct DivNode<I1, I2, O>
where
    I1: Clone,
    I2: Clone,
{
    state: BinOpState<I1, I2>,

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
            state: BinOpState::None,
            input_1: Input::new(),
            input_2: Input::new(),
            output_1: Output::new(change_observer),
        }
    }

    fn handle_1(&mut self, v: I1) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I1(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Division should happen pairwise.".into(),
                })
            }
            BinOpState::I2(i) => {
                let out = v / i.clone();
                self.state = BinOpState::None;
                self.output_1.clone().send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I1(v),
        }
        Ok(())
    }

    fn handle_2(&mut self, v: I2) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I2(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Division should happen pairwise.".into(),
                })
            }
            BinOpState::I1(i) => {
                let out = i.clone() / v;
                self.state = BinOpState::None;
                self.output_1.clone().send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I2(v),
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
    handle_sequentially!(input_1, input_2, handle_1, handle_2);
}

#[test]
fn should_div_132() -> Result<(), UpdateError> {
    let change_observer = ChangeObserver::new();

    let mut div: DivNode<f32, f32, f32> = DivNode::new(Some(&change_observer));
    let mock_output = flowrs::connection::Edge::new();
    flowrs::connection::connect(div.output_1.clone(), mock_output.clone());
    div.input_1.send(15.0)?;
    div.input_2.send(2.0)?;
    div.on_update()?;
    div.on_update()?;

    let expected = 7.5;
    let actual = mock_output.next()?;
    Ok(assert!(expected == actual))
}

#[test]
fn should_serialize_deserialize() -> Result<(), UpdateError> {
    let change_observer = ChangeObserver::new();

    let mut div: DivNode<i32, i32, i32> = DivNode::new(Some(&change_observer));
    div.input_1.send(2)?;
    div.on_update()?;

    let expected = r#"{"state":{"I1":2},"input_1":null,"input_2":null,"output_1":null}"#;
    let actual = serde_json::to_string(&div).unwrap();

    assert_eq!(expected, actual);

    let res = serde_json::from_str::<DivNode<i32, i32, i32>>(expected);
    let expected;
    match res {
        Ok(val) => expected = val,
        Err(e) => panic!("{}", e),
    }
    let actual = div.state;

    assert_eq!(
        serde_json::to_string(&expected.state).unwrap(),
        serde_json::to_string(&actual).unwrap()
    );
    Ok(())
}
