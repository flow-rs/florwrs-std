use flowrs::{node::{Node, UpdateError, ChangeObserver}, connection::{Input, Output, connect}};

//use flowrs_derive::RuntimeConnectable;

pub trait MergeWindow<T> {
    fn update(&mut self, t: T) -> Option<Vec<T>>;
}

pub struct CountWindow<T> {
    cur_vec: Vec<T>,
    max_elements: usize
}

impl<T> CountWindow<T> {
    pub fn new(max_elements: usize) -> Self {
        Self {
            cur_vec: Vec::new(),
            max_elements: max_elements
        }
    }
}

impl<T> MergeWindow<T> for CountWindow<T> {
    fn update(&mut self, t: T) -> Option<Vec<T>> {
        self.cur_vec.push(t);
        if self.cur_vec.len() >= self.max_elements {
            Some(self.cur_vec.drain(..).collect())
        } else {
            None
        }
    }
}

//#[derive(RuntimeConnectable)]
pub struct MergeToVecNode<T, W> {
    
    window: W,
    
    //#[output]
    pub output: Output<Vec<T>>,
    
    //#[input]
    pub input: Input<T>,
}

impl<T, W> MergeToVecNode<T, W> 
where W: MergeWindow<T>{
    pub fn new(change_observer: Option<&ChangeObserver>, window: W) -> Self{
        Self {
            input: Input::new(),
            output: Output::new(change_observer),
            window: window
        }
    }
}

impl<T, W> Node for MergeToVecNode<T, W> 
where T: Send, W: Send + MergeWindow<T>{

    fn on_update(&mut self) -> Result<(), UpdateError> {
        
        if let core::result::Result::Ok(element) = self.input.next() {
            if let Some(vec) = self.window.update(element) {
                if let Err(err) = self.output.send(vec){
                    UpdateError::Other(err.into());
                }
            }
        }
        Ok(())
    }
}

//#[derive(RuntimeConnectable)]
pub struct SplitVecNode<T> {
    //#[output]
    pub output: Output<T>,
    
    //#[input]
    pub input: Input<Vec<T>>,
}

impl<T> SplitVecNode<T> {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self{
        Self {
            input: Input::new(),
            output: Output::new(change_observer)
        }
    }
}

impl<T> Node for SplitVecNode<T> 
where T: Send{

    fn on_update(&mut self) -> Result<(), UpdateError> {
        
        if let core::result::Result::Ok(vec) = self.input.next() {
            for el in vec {
                if let Err(err) = self.output.send(el){
                    UpdateError::Other(err.into());
                }
            }
        }
        Ok(())
    }
}

#[test]
fn test_to_and_from_vec_with_count_window() {
    
    let num_elements: usize = 4;

    let mut merge_to_vec_node_1: MergeToVecNode<usize, CountWindow<usize>> = MergeToVecNode::new(None, CountWindow::new(num_elements)); 
    let mut merge_to_vec_node_2: MergeToVecNode<usize, CountWindow<usize>> = MergeToVecNode::new(None, CountWindow::new(num_elements / 2)); 
    let mut split_vec_node: SplitVecNode<usize> = SplitVecNode::new(None);

    let e: flowrs::connection::Edge<Vec<usize>> = flowrs::connection::Edge::new();
   
    connect(merge_to_vec_node_1.output.clone(), split_vec_node.input.clone());
    connect(split_vec_node.output.clone(), merge_to_vec_node_2.input.clone());
    connect(merge_to_vec_node_2.output.clone(), e.clone());
    
    for i in 0..num_elements {
        let _ = merge_to_vec_node_1.input.send(i);
        merge_to_vec_node_1.on_update();
    }

    split_vec_node.on_update();

    for i in 0..num_elements {
        merge_to_vec_node_2.on_update();
    }
    
    let should_be_1: Vec<usize> = (0..=num_elements / 2 - 1).collect();
    let should_be_2: Vec<usize> = (num_elements / 2..=num_elements-1).collect();

    let out_1 = e.next().expect("not enough vectors.");
    let out_2 = e.next().expect("not enough vectors.");
        
    assert_eq!(out_1, should_be_1);
    assert_eq!(out_2, should_be_2);

}