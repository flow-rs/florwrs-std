use flowrs::{node::{Node, UpdateError, ChangeObserver}, connection::{Input, Output, connect}};

use flowrs_derive::RuntimeConnectable;

use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable)]
pub struct ToJsonStringNode<T> {
    #[output]
    pub output: Output<String>,
    
    #[input]
    pub input: Input<T>,
}

impl<T> ToJsonStringNode<T> 
where T: Serialize {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for ToJsonStringNode<T>
where T: Serialize + Send {

    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next() {
            
            let res = serde_json::to_string(&input);
            match res {
                Ok(json_str) => {
                    _ = self.output.send(json_str);
                    return Ok(())
                },
                Err(err) => {
                    return Err(UpdateError::Other(err.into()));
                }
            }
        }
        Ok(())
    }
}

#[derive(RuntimeConnectable)]
pub struct FromJsonStringNode<T> {
    #[output]
    pub output: Output<T>,

    #[input]
    pub input: Input<String>,
}

impl<T> FromJsonStringNode<T>
    where T: for<'a> Deserialize<'a> + Send {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
            input: Input::new()
        }
    }
}

impl<T> Node for FromJsonStringNode<T> 
    where T: for<'a> Deserialize<'a> + Send {
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next() {
            
            let res = serde_json::from_str(input.as_str());
            match res {
                Ok(obj) => {
                    _ = self.output.send(obj);
                    return Ok(())
                },
                Err(err) => {
                    return Err(UpdateError::Other(err.into()));
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_to_json_and_from_string() {
    
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    struct TestStruct {
        a: i32,
        b: i32,
    }

    let inp = TestStruct{a:42, b: -42};

    let mut to_string_node: ToJsonStringNode<TestStruct> = ToJsonStringNode::new(None);
    let mut from_string_node: FromJsonStringNode<TestStruct> = FromJsonStringNode::new(None);

    let e: flowrs::connection::Edge<TestStruct> = flowrs::connection::Edge::new();
   
    connect(to_string_node.output.clone(), from_string_node.input.clone());
    connect(from_string_node.output.clone(), e.clone());
    
    let _ = to_string_node.input.send(inp.clone());
    
    let _ = to_string_node.on_update();
    let _ = from_string_node.on_update();

    let out = e.next().expect("");
    
    assert_eq!(out,inp);

}


#[test]
fn test_from_json_string_error() {
    
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    struct TestStruct {
        a: i32,
        b: i32,
    }
    
    let mut from_string_node: FromJsonStringNode<TestStruct> = FromJsonStringNode::new(None);

    let e: flowrs::connection::Edge<TestStruct> = flowrs::connection::Edge::new();
   
    connect(from_string_node.output.clone(), e.clone());

    let _ = from_string_node.input.send("{a:\"TEXT\"}}".to_string());

    let res = from_string_node.on_update();
    match res {
        Err(_) => assert!(true),
        Ok(_) => assert!(false)
    }
}