use flowrs::{node::{Node, ChangeObserver, InitError, ReadyError, ShutdownError, UpdateError}, connection::{Input, Output}};
use flowrs_derive::RuntimeConnectable;

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
    fn stop(&self);
}

pub struct Timer
{
    own_thread: bool,
    cond_var: Arc<(Mutex<bool>, Condvar)>,
}

impl TimerStrategy for Timer {
    
    fn start<F>(&self, every: Duration, mut closure: F)
    where F: 'static + FnMut() + Send {

        let pair = self.cond_var.clone();

        println!("{:?}", every);
       
    	let mut timer_closure = move || {        

            loop {

                (closure)();
                
                let (lock, cvar) = &*pair;
                
                let result = cvar.wait_timeout_while(
                    lock.lock().unwrap(),
                    every,
                    |&mut run| run,
                ).unwrap();  

                println!("WAKE UP");

                if !result.1.timed_out() {
                    println!("SHUTDOWN");
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

    fn stop(&self){
        println!("{:?} STOP", std::thread::current().id());

        let (lock, cvar) = &*self.cond_var;
        let mut run = lock.lock().unwrap();
        *run = false;
        cvar.notify_one();
    }

}

impl Timer {
    pub fn new(own_thread: bool) -> Self {
        Self {
            own_thread: own_thread,  
            cond_var: Arc::new((Mutex::new(true), Condvar::new())),
        }
    }
}


#[derive(RuntimeConnectable)]
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
    where T : TimerStrategy  + Send {
    pub fn new(name: &str, change_observer: &ChangeObserver, timer: T) -> Self {
        Self {
            name: name.into(), 
            config_input: Input::new(), 
            token_output: Output::new(change_observer),
            timer: timer
        }
    }
}

impl<T> Node for TimerNode<T>
    where T : TimerStrategy + Send + 'static{

    fn name(&self) -> &str {
        &self.name
    }

    fn on_init(&self) -> Result<(), InitError> {
        Ok(())
    }

    fn on_ready(&self) -> Result<(), ReadyError> {
        Ok(())
    }

    fn on_shutdown(&self) -> Result<(), ShutdownError> {
        println!("{:?} ON_SHUTDOWN", std::thread::current().id());

        self.timer.stop();
        Ok(())
    }

    fn update(&self) -> Result<(), UpdateError> {
        if let Ok(config) = self.config_input.next_elem() {
            
            let mut token_output_clone = self.token_output.clone();
            self.timer.start(config.duration, move ||{
                println!("{:?} TIMER TICK", std::thread::current().id());
                let _ = token_output_clone.send(TimerNodeToken{});
            });
        }
        Ok(())        
    }
}