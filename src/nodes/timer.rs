use flowrs::{node::{Node, State, ChangeObserver, InitError, ReadyError, ShutdownError, UpdateError, UpdateController}, connection::{Input, Output}};
use flowrs_derive::{Connectable};

use std::thread;
use std::sync::{Condvar, Mutex, Arc};
use core::time::Duration;

#[derive(Clone)]
pub struct TimerNodeConfig {
   pub duration: Duration
}

#[derive(Clone, Debug)]
pub struct TimerNodeToken {
}

pub trait TimerStrategy {
    fn start<F>(&self, every: Duration, closure: F) where F: 'static + FnMut() + Send;
    fn update_controller(&self) -> Arc<Mutex<dyn UpdateController>>;
}

struct WaitTimerUpdateController {
    cond_var: Arc<(Mutex<bool>, Condvar)>
}

impl WaitTimerUpdateController {
    
    pub fn new(cond_var: Arc<(Mutex<bool>, Condvar)>) -> Self {
        Self{           
            cond_var: cond_var
        }
    }
}

impl UpdateController for WaitTimerUpdateController {
    fn cancel(&mut self) {
        let (lock, cvar) = &*self.cond_var;
        let mut run = lock.lock().unwrap();
        *run = false;
        cvar.notify_one();
    }
}

pub struct WaitTimer {
    own_thread: bool,
    cond_var: Arc<(Mutex<bool>, Condvar)>,
}

impl TimerStrategy for WaitTimer {
    
    fn start<F>(&self, every: Duration, mut closure: F)
    where F: 'static + FnMut() + Send {

        let pair = self.cond_var.clone();
       
    	let mut timer_closure = move || {        

            loop {

                (closure)();
                
                let (lock, cvar) = &*pair;
                
                let result = cvar.wait_timeout_while(
                    lock.lock().unwrap(),
                    every,
                    |&mut run| run,
                ).unwrap();  

                if !result.1.timed_out() {
                    println!("TIMER SHUTDOWN");
                    break;
                } 
        
            }
        };

        if self.own_thread {
            thread::spawn(timer_closure);
        }
        else {
            (timer_closure)();
        }


    }

    fn update_controller(&self) -> Arc<Mutex<dyn UpdateController>> {
        Arc::new(Mutex::new(WaitTimerUpdateController::new(self.cond_var.clone())))
    }

}

impl WaitTimer {
    pub fn new(own_thread: bool) -> Self {
        Self {
            own_thread: own_thread,  
            cond_var: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }
}

#[derive(Connectable)]
pub struct TimerNode<T>
{
    name: String,
    timer: T,

    #[input]
    pub config_input: Input<TimerNodeConfig>,

    #[output]
    pub token_output: Output<TimerNodeToken>,
}

impl<T> TimerNode<T>
    where T : TimerStrategy {
    pub fn new(name: &str, timer: T, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            name: name.into(), 
            config_input: Input::new(), 
            token_output: Output::new(change_observer),
            timer: timer
        }
    }
}

impl<T> Node for TimerNode<T>
    where T : TimerStrategy + Send {

    fn name(&self) -> &str {
        &self.name
    }

    fn on_update(&self) -> Result<(), UpdateError> {

        //println!("{:?} TIMER UPDATE 0", std::thread::current().id());
        if let Ok(config) = self.config_input.next_elem() {
            //println!("{:?} TIMER UPDATE 1", std::thread::current().id());
            
            let mut token_output_clone = self.token_output.clone();
            
            self.timer.start(config.duration, move ||{
                println!("                                                  {:?} TIMER TICK 1", std::thread::current().id());
                let res = token_output_clone.send(TimerNodeToken{});
                println!("                                                  {:?} TIMER TICK 2 {:?}", std::thread::current().id(), res);
               
            });
        }
        Ok(())        
    }

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        Some(self.timer.update_controller())
    }
}