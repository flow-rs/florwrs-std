use flowrs::node::{ ChangeObserver, Node, InitError, ReadyError, ShutdownError, UpdateError};
use flowrs::connection::{Input, Output};
use flowrs_derive::RuntimeConnectable;

use std::fs::File;

#[derive(RuntimeConnectable)]
pub struct DummyNode {
    name: String,

    #[input]
    pub input_1: Input<i32>,
    #[output]
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
}

#[cfg(test)]
mod sched {
    
    use flowrs::{execution::{Executor, StandardExecutor}, scheduler::{RoundRobinScheduler}, node::{Context, ChangeObserver, InitError, ReadyError, ShutdownError, UpdateError}, flow::Flow, version::Version, sched::node_updater::MultiThreadedNodeUpdater};
    use flowrs::connection::{connect, Edge, Input};
    use serde_json::Value;

    use std::{thread, sync::mpsc, time::Duration, collections::HashMap};
    use crate::sched::test_sched::DummyNode;

    #[test]
    fn test_executor() {
 
        let (sender, receiver) = mpsc::channel();
        
        let change_observer = ChangeObserver::new();  

        let n1: DummyNode = DummyNode::new("node_1", false, Some(&change_observer));
        let mock_input = Input::<i32>::new();        
        connect(n1.output_1.clone(), mock_input.clone());


        let mut flow: Flow<String> = Flow::new_empty("flow_1", Version::new(1,0,0));

        let _ = n1.input_1.send(1);
      
        
        flow.add_node(n1, "first".into());

        let thread_handle = thread::spawn( move || {
        
            let num_workers = 4;
            let mut executor = StandardExecutor::new(change_observer);
            
            let _ = sender.send(executor.controller());

            let _ = executor.run(flow, RoundRobinScheduler::new(), MultiThreadedNodeUpdater::new(num_workers));
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
       let mut flow: Flow<String> = Flow::new_empty("flow_1", Version::new(1,0,0));
      
       flow.add_node(n1, "first".into());
       flow.add_node(n2, "second".into());

       let mut ex = StandardExecutor::new(change_observer);

       match ex.run(flow, RoundRobinScheduler::new(), MultiThreadedNodeUpdater::new(1)) {
        Ok(_) => assert!(false),
        Err(_) => assert!(true)
       }

    }
} 