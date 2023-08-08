#[cfg(test)]
mod nodes {
    use std::{thread, time::Duration, sync::mpsc};
        
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver},
        sched::{version::Version, flow::Flow, executor::{Executor, StandardExecutor}, scheduler::RoundRobinScheduler},
    };

    use flowrs_std::{value::ValueNode, timer::{TimerNodeConfig, TimerNode, TimerNodeToken, WaitTimer}, debug::DebugNode};
    
    #[test]
    fn test() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 
      
        let node_1 = ValueNode::new(
            "value_node", 
            &change_observer, 
            TimerNodeConfig {duration: core::time::Duration::from_secs(1) }
        );
        
        let node_2 = TimerNode::new("timer_node", &change_observer, WaitTimer::new(false));

        let node_3 = DebugNode::<TimerNodeToken>::new("debug_node", &change_observer);

        let mock_input = Edge::new();

        connect(node_1.output.clone(), node_2.config_input.clone());
        connect(node_2.token_output.clone(), node_3.input.clone());
        connect(node_3.output.clone(), mock_input.clone());

        let mut flow = Flow::new("flow_1", Version::new(1,0,0), Vec::new());
        flow.add_node(node_1);
        flow.add_node(node_2);
        flow.add_node(node_3);
        
        let (sender, receiver) = mpsc::channel();
        let thread_handle = thread::spawn( move || {
        
            let num_workers = 4;
            let mut executor = StandardExecutor::new(num_workers, change_observer);
            let mut scheduler = RoundRobinScheduler::new();

            let _ = sender.send(executor.controller());

            executor.run(flow, scheduler);
        });

        let controller = receiver.recv().unwrap();

        thread::sleep(Duration::from_secs(5));

        println!("                                      ----> {:?} CANCEL", std::thread::current().id());

        controller.lock().unwrap().cancel();
       
        thread_handle.join().unwrap();


        /*
        node_1.on_ready();     
        node_2.update(); 

        thread::sleep(Duration::from_secs(1));  
        node_3.update();

        thread::sleep(Duration::from_secs(1));  
        node_3.update();
         */


    }
}
