use flowrs::{connection::{Input}, node::{Node, UpdateError}};
use flowrs_std::timer::TimerNodeToken;
use flowrs_derive::Connectable;
use std::sync::mpsc::{Sender};

#[derive(Connectable)]
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
        
        if let Ok(input) = self.input.next_elem() {            
            let _res = self.sender.send(input);
        }
        Ok(())
    }
}

#[cfg(test)]
mod nodes {
    use std::{thread, time::Duration, sync::mpsc::channel};
        
    use flowrs::{
        connection::{connect},
        node::{ChangeObserver},
        sched::{version::Version, flow::Flow, execution::{Executor, StandardExecutor}, scheduler::RoundRobinScheduler},
    };

    use flowrs_std::{value::ValueNode, timer::{TimerNodeConfig, TimerNode, TimerNodeToken, WaitTimer, PollTimer, TimerStrategy}, debug::DebugNode};
    
    use crate::nodes::test_timer::ReportNode;

    fn timer_test_with<T: TimerStrategy + Send + 'static>(num_workers: usize, timer: T) {

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
        
            let mut executor = StandardExecutor::new(num_workers, change_observer);
            let scheduler = RoundRobinScheduler::new();

            let _ = controller_sender.send(executor.controller());

            executor.run(flow, scheduler);
        });

        let controller = controller_receiver.recv().unwrap();


        thread::sleep(Duration::from_secs(sleep_seconds));

        //println!("                                      ----> {:?} CANCEL", std::thread::current().id());

        controller.lock().unwrap().cancel();
       
        thread_handle.join().unwrap();

        let num_iters = receiver.iter().count();

        assert_eq!(num_iters as u64, sleep_seconds / timer_interval_seconds);
    }

    #[test]
    fn test() {

        timer_test_with(4, WaitTimer::new(true));
        
        timer_test_with(4, WaitTimer::new(false));

        timer_test_with(0, WaitTimer::new(true));
    
        timer_test_with(0, PollTimer::new());

        // This combination cannot work, since the single execution thread is blocked by the timer. 
        // timer_test_with(0, WaitTimer::new(false));

        // This combination cannot work, since with multiple workers, a the execution unit sleeps without written outputs.
        // timer_test_with(4, TimeSliceTimer::new());
    }
}
