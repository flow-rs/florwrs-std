use super::binops::BinOpState;
use crate::handle_sequentially;
use flowrs::{
    comm::communication::NodeCommunicator,
    connection::{Input, Output},
    node::{Node, UpdateError},
    //RuntimeConnectable,
};
use std::{fmt, ops::Add, str::FromStr};

//#[derive(RuntimeConnectable)]
pub struct AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
{
    state: BinOpState<I1, I2>,

    //#[input]
    pub input_1: Input<I1>,
    //#[input]
    pub input_2: Input<I2>,
    //#[output]
    pub output_1: Output<O>,
}

impl<I1, I2, O> AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I1: Add<I2, Output = O>,
    I1: Send + 'static,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    I2: Send + 'static,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + 'static,
{
    // pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
    //     Self {
    //         state: BinOpState::None,
    //         input_1: Input::new(),
    //         input_2: Input::new(),
    //         output_1: Output::new(change_observer),
    //     }
    // }

    pub fn new(
        input_1_communicator: NodeCommunicator<I1>,
        input_2_communicator: NodeCommunicator<I2>,
        output_communicator: NodeCommunicator<O>,
    ) -> Self {
        Self {
            state: BinOpState::None,
            input_1: Input::new(input_1_communicator),
            input_2: Input::new(input_2_communicator),
            output_1: Output::new(output_communicator),
        }
    }

    fn handle_1(&mut self, v: I1) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I1(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Addition should happen pairwise.".into(),
                })
            }
            BinOpState::I2(i) => {
                let out = v + i.clone();
                self.state = BinOpState::None;
                self.output_1.send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I1(v),
        }
        Ok(())
    }

    fn handle_2(&mut self, v: I2) -> Result<(), UpdateError> {
        match &self.state {
            BinOpState::I2(_) => {
                return Err(UpdateError::SequenceError {
                    message: "Addition should happen pairwise.".into(),
                })
            }
            BinOpState::I1(i) => {
                let out = i.clone() + v;
                self.state = BinOpState::None;
                self.output_1.send(out)?;
            }
            BinOpState::None => self.state = BinOpState::I2(v),
        }
        Ok(())
    }
}

impl<I1, I2, O> Node for AddNode<I1, I2, O>
where
    I1: Clone,
    I1: fmt::Debug,
    I1: FromStr,
    I1: Add<I2, Output = O>,
    I1: Send + 'static,
    I2: Clone,
    I2: fmt::Debug,
    I2: FromStr,
    I2: Send + 'static,
    O: Clone,
    O: fmt::Debug,
    O: FromStr,
    O: Send + 'static,
{
    handle_sequentially!(input_1, input_2, handle_1, handle_2);

    fn set_execution_mode(
        &mut self,
        _mode: flowrs::exec::execution_mode::ExecutionMode,
    ) -> flowrs::exec::execution_mode::ExecutionMode {
        flowrs::exec::execution_mode::ExecutionMode::Continuous
    }

    fn get_execution_mode(&self) -> flowrs::exec::execution_mode::ExecutionMode {
        flowrs::exec::execution_mode::ExecutionMode::Continuous
    }

    fn on_init(&mut self) -> Result<(), flowrs::node::InitError> {
        Ok(())
    }

    fn on_ready(&mut self) -> Result<(), flowrs::node::ReadyError> {
        Ok(())
    }

    fn on_shutdown(&mut self) -> Result<(), flowrs::node::ShutdownError> {
        Ok(())
    }

    fn update_controller(&self) -> Option<Box<dyn flowrs::node::UpdateController>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use flowrs::{comm::thread_communicator::ThreadCommunicator, node::UpdateError};

    use super::*;

    #[test]
    fn should_add_132() -> Result<(), UpdateError> {
        // Set up communicators
        let i1_comm = NodeCommunicator::ThreadComm(
            ThreadCommunicator::<i32>::new().expect("should construct"),
        );
        let i2_comm = NodeCommunicator::ThreadComm(
            ThreadCommunicator::<i32>::new().expect("should construct"),
        );
        let o_comm = NodeCommunicator::ThreadComm(
            ThreadCommunicator::<i32>::new().expect("should construct"),
        );
        // Create AddNode
        let mut add: AddNode<i32, i32, i32> = AddNode::new(i1_comm, i2_comm, o_comm);
        //flowrs::connection::connect(add.output_1.clone(), mock_output.clone());
        add.input_1.send(1)?;
        add.input_2.send(2)?;
        add.on_update()?;

        let expected = 3;
        let actual = add.output_1.next()?;
        Ok(assert_eq!(expected, actual))
    }

    // #[test]
    // fn should_serialize_deserialize() -> Result<(), UpdateError> {
    //     let change_observer = ChangeObserver::new();

    //     let mut add: AddNode<i32, i32, i32> = AddNode::new(Some(&change_observer));
    //     add.input_1.send(2)?;
    //     add.on_update()?;

    //     let expected = r#"{"state":{"I1":2},"input_1":null,"input_2":null,"output_1":null}"#;
    //     let actual = serde_json::to_string(&add).unwrap();

    //     assert_eq!(expected, actual);

    //     let res = serde_json::from_str::<AddNode<i32, i32, i32>>(expected);
    //     let expected;
    //     match res {
    //         Ok(val) => expected = val,
    //         Err(e) => panic!("{}", e),
    //     }
    //     let actual = add.state;

    //     assert_eq!(
    //         serde_json::to_string(&expected.state).unwrap(),
    //         serde_json::to_string(&actual).unwrap()
    //     );
    //     Ok(())
    // }
}
