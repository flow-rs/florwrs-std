use flowrs::{node::{ ChangeObserver, Node, InitError, ReadyError, ShutdownError, UpdateError}};
use flowrs::connection::{Input, Output};
use flowrs_derive::RuntimeConnectable;


use std::fs::File;

#[derive(RuntimeConnectable)]
pub struct DummyNode {
    name: String,

    pub input_1: Input<i32>,
    pub output_1: Output<i32>,
    err_on_init: bool
}

impl DummyNode {
    pub fn new(name: &str, err_on_init: bool, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            name: name.into(),
            input_1: Input::new(),
            output_1: Output::new(change_observer),
            err_on_init: err_on_init
        }
    }
}

impl Node for DummyNode {
    fn on_init(&self)-> Result<(), InitError> {
        
        if self.err_on_init {
            let _file = File::open("").map_err(|err| InitError::Other(err.into()))?;
        }
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod sched {
    
    use flowrs::{execution::{Executor, StandardExecutor}, scheduler::{RoundRobinScheduler}, node::{Context, State, ChangeObserver, InitError, ReadyError, ShutdownError, UpdateError}, flow::Flow, version::Version};
    use flowrs::connection::{connect, Edge, Input};
    use serde_json::Value;

    use std::{thread, sync::mpsc, time::Duration};
    use crate::sched::test_sched::DummyNode;

    #[test]
    fn test_executor() {
 
        let (sender, receiver) = mpsc::channel();
        
        let change_observer = ChangeObserver::new();  

        let n1: DummyNode = DummyNode::new("node_1", false, Some(&change_observer));
        let mock_input = Input::<i32>::new();        
        connect(n1.output_1.clone(), mock_input.clone());


        let mut flow = Flow::new("flow_1", Version::new(1,0,0), Vec::new());

        let _ = n1.input_1.send(1);
      
        
        flow.add_node(n1);

        let thread_handle = thread::spawn( move || {
        
            let num_threads = 4;
            let mut executor = StandardExecutor::new(num_threads, change_observer);
            let scheduler = RoundRobinScheduler::new();

            let _ = sender.send(executor.controller());

            let _ = executor.run(flow, scheduler);
        });

        let controller = receiver.recv().unwrap();

        thread::sleep(Duration::from_secs(3));

        println!("CANCEL");

        controller.lock().unwrap().cancel();
       
        thread_handle.join().unwrap();

        println!("DONE");


        //println!("Has next: {}",  mock_output.has_next());

    }


    #[test]
    fn test_error_behavior() {

        let change_observer = ChangeObserver::new();  

       let n1: DummyNode = DummyNode::new("node_1", true, Some(&change_observer));
       let n2: DummyNode = DummyNode::new("node_2", true, Some(&change_observer));
       let mut flow = Flow::new("flow_1", Version::new(1,0,0),Vec::new());
      
       flow.add_node(n1);
       flow.add_node(n2);

       let mut ex = StandardExecutor::new(1, change_observer);

       match ex.run(flow, RoundRobinScheduler::new()) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true)
       }

    }
} 