use std::{
    any::Any,
    fmt,
    rc::Rc,
    sync::{
        mpsc::{channel, Receiver, RecvError, SendError, Sender, TryRecvError},
        Arc, Mutex,
    },
};

#[derive(Debug)]
pub enum ConnectError<I> {
    SendErr(SendError<I>),
    RecvErr(RecvError),
    TryRecvErr(TryRecvError),
    ChanErr(ChannelError),
}

#[derive(Debug, Clone)]
pub struct ChannelError {
    index: usize,
    size: usize,
}

impl<I> From<SendError<I>> for ConnectError<I> {
    fn from(value: SendError<I>) -> Self {
        ConnectError::SendErr(value)
    }
}

impl<I> From<RecvError> for ConnectError<I> {
    fn from(value: RecvError) -> Self {
        ConnectError::RecvErr(value)
    }
}

impl<I> From<TryRecvError> for ConnectError<I> {
    fn from(value: TryRecvError) -> Self {
        ConnectError::TryRecvErr(value)
    }
}

impl<I> From<ChannelError> for ConnectError<I> {
    fn from(value: ChannelError) -> Self {
        ConnectError::ChanErr(value)
    }
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This Node has not enough inputs. Attempted write on input {} while this node only has {} inputs.", self.index, self.size)
    }
}

#[derive(Debug)]
pub struct Edge<I> {
    sender: Sender<I>,
    receiver: Option<Receiver<I>>,
}

impl<I> Clone for Edge<I> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: None,
        }
    }
}

impl<I> Edge<I> {
    pub fn new() -> Self {
        let (sender, receiver) = channel();
        Self {
            sender,
            receiver: Some(receiver),
        }
    }

    pub fn send(&self, elem: I) -> Result<(), ConnectError<I>> {
        Ok(self.sender.send(elem)?)
    }

    pub fn next_elem(&mut self) -> Result<I, ConnectError<I>> {
        Ok(self
            .receiver
            .as_mut()
            .expect("Only the Node that created this edge can receive from it.")
            .try_recv()?)
    }
}

pub type Input<I> = Edge<I>;

#[derive(Clone, Debug)]
pub struct Output<T>(Arc<Mutex<Option<Edge<T>>>>);

impl<O> Output<O> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    pub fn send(&mut self, elem: O) -> Result<(), ConnectError<O>> {
        self.0
            .lock()
            .unwrap()
            .as_mut()
            .expect("You attemnted to send to an output where no succesor Node is connected.")
            .send(elem)
    }

    pub fn set(&mut self, edge: Edge<O>) {
        let _ = self.0.lock().unwrap().insert(edge);
    }
}

pub fn connect<I>(mut lhs: Output<I>, rhs: Input<I>) {
    lhs.set(rhs)
}

pub trait RuntimeConnectable {
    fn input_at(&self, index: usize) -> Rc<dyn Any>;
    fn output_at(&self, index: usize) -> Rc<dyn Any>;
}
