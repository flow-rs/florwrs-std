use std::{any::Any, sync::Arc};

use flowrs::RuntimeConnectable;
use flowrs::{
    connection::{connect, Input, Output, RuntimeConnectable},
    node::{ChangeObserver, Node, UpdateError},
};
use serde::{Deserialize, Serialize};

pub trait MergeWindow<T> {
    fn update(&mut self, t: T) -> Option<Vec<T>>;
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CountWindow<T> {
    cur_vec: Vec<T>,
    max_elements: usize,
}

impl<T> CountWindow<T> {
    pub fn new(max_elements: usize) -> Self {
        Self {
            cur_vec: Vec::new(),
            max_elements: max_elements,
        }
    }
}

impl<T> MergeWindow<T> for CountWindow<T> {
    fn update(&mut self, t: T) -> Option<Vec<T>> {
        self.cur_vec.push(t);
        if self.cur_vec.len() >= self.max_elements {
            return Some(self.cur_vec.drain(..).collect());
        } else {
            return None;
        }
    }
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct MergeToVecNode<I, O, W> {
    window: W,

    #[input]
    pub input: Input<I>,

    #[output]
    pub output: Output<O>,
}

impl<I, O, W> MergeToVecNode<I, O, W>
where
    W: MergeWindow<I>,
{
    pub fn new(change_observer: Option<&ChangeObserver>, window: W) -> Self {
        Self {
            input: Input::new(),
            output: Output::new(change_observer),
            window: window,
        }
    }
}

impl<I, O, W> Node for MergeToVecNode<I, O, W>
where
    I: Send,
    O: Send + From<Vec<I>>,
    W: Send + MergeWindow<I>,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(element) = self.input.next() {
            if let Some(vec) = self.window.update(element) {
                self.output
                    .send(vec.into())
                    .map_err(|e| UpdateError::Other(e.into()))?;
            }
        }
        Ok(())
    }
}

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct SplitVecNode<T> {
    #[output]
    pub output: Output<T>,

    #[input]
    pub input: Input<Vec<T>>,
}

impl<T> SplitVecNode<T> {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            input: Input::new(),
            output: Output::new(change_observer),
        }
    }
}

impl<T> Node for SplitVecNode<T>
where
    T: Send,
{
    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let core::result::Result::Ok(vec) = self.input.next() {
            for el in vec {
                self.output
                    .send(el)
                    .map_err(|e| UpdateError::Other(e.into()))?;
            }
        }
        Ok(())
    }
}

#[test]
fn test_to_and_from_vec_with_count_window() -> Result<(), anyhow::Error> {
    let num_elements: usize = 4;

    let mut merge_to_vec_node_1: MergeToVecNode<usize, Vec<usize>, CountWindow<usize>> =
        MergeToVecNode::new(None, CountWindow::new(num_elements));
    let mut merge_to_vec_node_2: MergeToVecNode<usize, Vec<usize>, CountWindow<usize>> =
        MergeToVecNode::new(None, CountWindow::new(num_elements / 2));
    let mut split_vec_node: SplitVecNode<usize> = SplitVecNode::new(None);

    let e: flowrs::connection::Edge<Vec<usize>> = flowrs::connection::Edge::new();

    connect(
        merge_to_vec_node_1.output.clone(),
        split_vec_node.input.clone(),
    );
    connect(
        split_vec_node.output.clone(),
        merge_to_vec_node_2.input.clone(),
    );
    connect(merge_to_vec_node_2.output.clone(), e.clone());

    for i in 0..num_elements {
        let _ = merge_to_vec_node_1.input.send(i);
        merge_to_vec_node_1.on_update()?;
    }

    split_vec_node.on_update()?;

    for _ in 0..num_elements {
        merge_to_vec_node_2.on_update()?;
    }

    let should_be_1: Vec<usize> = (0..=num_elements / 2 - 1).collect();
    let should_be_2: Vec<usize> = (num_elements / 2..=num_elements - 1).collect();

    let out_1 = e.next().expect("not enough vectors.");
    let out_2 = e.next().expect("not enough vectors.");

    assert_eq!(out_1, should_be_1);
    assert_eq!(out_2, should_be_2);
    Ok(())
}

#[test]
fn test_output() -> Result<(), anyhow::Error> {
    let num_elements: usize = 4;

    let merge_to_vec_node_1: MergeToVecNode<usize, Vec<usize>, CountWindow<usize>> =
        MergeToVecNode::new(None, CountWindow::new(num_elements));

    let binding = merge_to_vec_node_1.output_at(0);
    let out_edge = binding.downcast_ref::<Output<Vec<usize>>>();

    assert!(out_edge.is_some());
    Ok(())
}

#[test]
fn test_any_output() -> Result<(), anyhow::Error> {
    #[derive(Clone)]
    pub struct FlowType(pub Arc<dyn Any + Send + Sync>);
    let num_elements: usize = 4;

    let merge_to_vec_node_1: MergeToVecNode<FlowType, FlowType, CountWindow<FlowType>> =
        MergeToVecNode::new(None, CountWindow::new(num_elements));

    let binding = merge_to_vec_node_1.output_at(0);
    let out_edge = binding.downcast_ref::<Output<FlowType>>();

    assert!(out_edge.is_some());
    Ok(())
}
