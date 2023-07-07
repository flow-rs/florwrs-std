use std::{fmt::Display, sync::Arc};

use serde::Deserialize;

use crate::Connectable;
use crate::{
    job::{Context, Job},
    log,
};

#[derive(Connectable, Deserialize)]
pub struct DebugNode<I, O>
where
    I: Sized,
    O: Sized,
{
    pub conn: Connection<I, O>,
    _context: Arc<Context>,
    name: String,
}

impl<I, O> DebugNode<I, O> {
    pub fn new(name: &str, context: Arc<Context>) -> Self {
        let conn = Connection::new(1);
        Self {
            conn,
            name: name.into(),
            _context: context,
        }
    }
}

impl<I, O> Job for DebugNode<I, O>
where
    I: Display + Clone,
    O: Clone,
{
    fn handle(&mut self) {
        print!("{} node prints: ", self.name);
        self.conn.input.iter().for_each(|c| {
            // Avoiding recv_timout since wasm can't access system time without JS bindings
            let msg = match c.try_recv() {
                Err(_) => format!("Nothing"),
                Ok(x) => format!("{}", x),
            };
            // Log to terminal and Browser console
            println!("{}", msg);
            log(msg.as_str());
        });
    }

    fn name(&self) -> &String {
        &self.name
    }

    fn init(&mut self) {
        ()
    }

    fn destory(&mut self) {
        ()
    }
}
