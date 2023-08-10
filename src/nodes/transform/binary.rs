use flowrs::{node::{Node, UpdateError, ChangeObserver}, connection::{Input, Output, connect}};

use flowrs_derive::RuntimeConnectable;

use serde::{Deserialize, Serialize};

//#[derive(RuntimeConnectable)]
pub struct ToBinaryNode<T> {
    //#[output]
    pub output: Output<Vec<u8>>,
    
    //#[input]
    pub input: Input<T>,
}

impl<T> ToBinaryNode<T> 
where T: Serialize {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for ToBinaryNode<T>
where T: Serialize + Send {

    fn on_update(&mut self) -> Result<(), UpdateError> {
        
        if let Ok(obj) = self.input.next() {
            let data = bincode::serialize(&obj).map_err(|e| UpdateError::Other(e.into()))?;
            self.output.send(data).map_err(|e| UpdateError::Other(e.into()))?;   
        }
        Ok(())
    }
}

//#[derive(RuntimeConnectable)]
pub struct FromBinaryNode<T> {
    //#[output]
    pub output: Output<T>,

    //#[input]
    pub input: Input<Vec<u8>>,
}

impl<T> FromBinaryNode<T>
    where T: for<'a> Deserialize<'a> + Send {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for FromBinaryNode<T> 
    where T: for<'a> Deserialize<'a> + Send {
    fn on_update(&mut self) -> Result<(), UpdateError> {

        if let Ok(data) = self.input.next() {
            let obj = bincode::deserialize(&data).map_err(|e| UpdateError::Other(e.into()))?;
            self.output.send(obj).map_err(|e| UpdateError::Other(e.into()))?;
        }
        Ok(())
    }
}

#[test]
fn test_to_and_from_binary() {
    
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    struct TestStruct {
        a: i32,
        b: i32,
    }

    let inp = TestStruct{a:42, b: -42};

    let mut to_binary_node: ToBinaryNode<TestStruct> = ToBinaryNode::new(None);
    let mut from_binary_node: FromBinaryNode<TestStruct> = FromBinaryNode::new(None);

    let e: flowrs::connection::Edge<TestStruct> = flowrs::connection::Edge::new();
   
    connect(to_binary_node.output.clone(), from_binary_node.input.clone());
    connect(from_binary_node.output.clone(), e.clone());
    
    let _ = to_binary_node.input.send(inp.clone());
    
    let _ = to_binary_node.on_update();
    let _ = from_binary_node.on_update();

    let out = e.next().expect("");
    
    assert_eq!(out,inp);

}
