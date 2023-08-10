use flowrs::{node::{Node, ChangeObserver, UpdateError, UpdateController}, connection::{Input, Output}};
use flowrs_derive::RuntimeConnectable;

use std::{time::Instant, thread, sync::{Condvar, Mutex, Arc}};
use core::time::Duration;

#[derive(Clone)]
pub struct TimerNodeConfig {
   pub duration: Duration
}

#[derive(Clone, Debug)]
pub struct TimerNodeToken {
}

pub trait TimerStrategy {
    fn start<F>(&mut self, every: Duration, closure: F) where F: 'static + FnMut() + Send;
    fn update(&mut self , _output: &mut Output<TimerNodeToken>) {}
    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {None}
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
    
    fn start<F>(&mut self, every: Duration, mut closure: F)
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
                    //println!("TIMER SHUTDOWN");
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

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        Some(Arc::new(Mutex::new(WaitTimerUpdateController::new(self.cond_var.clone()))))
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

pub struct PollTimer {
    every: Duration,
    last_tick: Instant
}

impl PollTimer {
    pub fn new() -> Self {
        Self {
            every: Duration::ZERO,//set later
            last_tick: Instant::now()
        }
    }
}

impl TimerStrategy for PollTimer {
    fn start<F>(&mut self, every: Duration, _closure: F) where F: 'static + FnMut() + Send {
        self.every = every;
        self.last_tick = Instant::now();
    }

    fn update(&mut self , output: &mut Output<TimerNodeToken>) {
        
        if self.last_tick.elapsed() >= self.every {
            let _res = output.send(TimerNodeToken {});
            self.last_tick = Instant::now();
            //println!("TICK");
        }
    
    }
}

#[derive(RuntimeConnectable)]
pub struct TimerNode<T>
{
    timer: T,

    #[input]
    pub config_input: Input<TimerNodeConfig>,

    #[output]
    pub token_output: Output<TimerNodeToken>,
}

impl<T> TimerNode<T>
    where T : TimerStrategy {
    pub fn new(timer: T, change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            config_input: Input::new(), 
            token_output: Output::new(change_observer),
            timer: timer
        }
    }
}

impl<T> Node for TimerNode<T>
    where T : TimerStrategy + Send {

    fn on_update(&mut self) -> Result<(), UpdateError> {

        if let Ok(config) = self.config_input.next_elem() {
            
            let mut token_output_clone = self.token_output.clone();
            
            self.timer.start(config.duration, move ||{
                //println!("                                                  {:?} TIMER TICK 1", std::thread::current().id());
                let res = token_output_clone.send(TimerNodeToken{});
                //println!("                                                  {:?} TIMER TICK 2 {:?}", std::thread::current().id(), res);
               
            });
        }

        self.timer.update(&mut self.token_output);

        Ok(())        
    }

    fn update_controller(&self) -> Option<Arc<Mutex<dyn UpdateController>>> {
        self.timer.update_controller()
    }
}