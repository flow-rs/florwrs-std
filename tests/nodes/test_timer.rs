use flowrs::{connection::{Input}, node::{Node, UpdateError}};
use flowrs_std::timer::TimerNodeToken;
use flowrs_derive::RuntimeConnectable;
use std::sync::mpsc::Sender;

#[derive(RuntimeConnectable)]
pub struct ReportNode {
    pub sender: Sender<TimerNodeToken>,

    #[input]
    pub input: Input<TimerNodeToken>,
}

impl ReportNode {
    pub fn new(sender: Sender<TimerNodeToken>) -> Self {
        Self {
            sender: sender,
            input: Input::new()
        }
    }
}

impl Node for ReportNode {
   
    fn on_update(&mut self) -> Result<(), UpdateError> {
        
        if let Ok(input) = self.input.next() {            
            let _res = self.sender.send(input);
        }
        Ok(())
    }
}

#[cfg(test)]
mod nodes {
    use std::{thread, time::Duration, sync::mpsc::channel};
        
    use flowrs::{
        connection::connect,
        node::ChangeObserver,
        sched::{version::Version, flow::Flow, execution::{Executor, StandardExecutor}, scheduler::RoundRobinScheduler, node_updater::{MultiThreadedNodeUpdater, SingleThreadedNodeUpdater, NodeUpdater}},
    };

    use flowrs_std::{value::ValueNode, timer::{TimerNodeConfig, TimerNode, TimerNodeToken, WaitTimer, PollTimer, TimerStrategy}, debug::DebugNode};
    
    use crate::nodes::test_timer::ReportNode;

    fn timer_test_with<T: TimerStrategy + Send + 'static, U: NodeUpdater + Drop + Send + 'static>(node_updater: U, timer: T) {

        let sleep_seconds = 5;
        let timer_interval_seconds = 1;

        let change_observer: ChangeObserver = ChangeObserver::new(); 
        let (sender, receiver) = channel::<TimerNodeToken>();
        
        let node_1 = ValueNode::new( 
            TimerNodeConfig {duration: core::time::Duration::from_secs(timer_interval_seconds) },
            Some(&change_observer)            
        );
        
        let node_2 = TimerNode::new(timer, Some(&change_observer));

        let node_3 = DebugNode::<TimerNodeToken>::new(Some(&change_observer));

        let node_4 = ReportNode::new(sender);

        connect(node_1.output.clone(), node_2.config_input.clone());
        connect(node_2.token_output.clone(), node_3.input.clone());
        connect(node_3.output.clone(), node_4.input.clone());

        let mut flow = Flow::new("flow_1", Version::new(1,0,0), Vec::new());
        flow.add_node(node_1);
        flow.add_node(node_2);
        flow.add_node(node_3);
        flow.add_node(node_4);

        let (controller_sender, controller_receiver) = channel();
        let thread_handle = thread::spawn( move || {
        
            let mut executor = StandardExecutor::new(change_observer);
            
            controller_sender.send(executor.controller()).expect("Controller sender cannot send.");

            executor.run(flow, RoundRobinScheduler::new(), node_updater).expect("Run failed.");
        });

        let controller = controller_receiver.recv().unwrap();


        thread::sleep(Duration::from_secs(sleep_seconds));

        //println!("                                      ----> {:?} CANCEL", std::thread::current().id());

        controller.lock().unwrap().cancel();
       
        thread_handle.join().unwrap();

        let num_iters = receiver.iter().count();

        let asserted_num_iters = sleep_seconds / timer_interval_seconds;

        //println!("{} {}", num_iters, asserted_num_iters.abs_diff(num_iters as u64));
        assert!(asserted_num_iters.abs_diff(num_iters as u64) <= 1);

    }

    #[test]
    fn test() {

        timer_test_with(MultiThreadedNodeUpdater::new(4), WaitTimer::new(true));
        
        timer_test_with(MultiThreadedNodeUpdater::new(4), WaitTimer::new(false));

        timer_test_with(SingleThreadedNodeUpdater::new(Some(100)), WaitTimer::new(true));
    
        timer_test_with(SingleThreadedNodeUpdater::new(Some(100)), PollTimer::new());

        // This combination cannot work, since the single execution thread is blocked by the timer. 
        // timer_test_with(SingleThreadedNodeUpdater::new(), WaitTimer::new(false));

        // This combination cannot work, since with multiple workers, a the execution unit sleeps without written outputs.
        // timer_test_with(MultiThreadedNodeUpdater::new(4), TimeSliceTimer::new());
    }
}
