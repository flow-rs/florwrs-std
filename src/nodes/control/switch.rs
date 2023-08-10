use flowrs::{node::{Node, UpdateError, ChangeObserver}, connection::{Input, Output, connect}};

use flowrs_derive::RuntimeConnectable;


#[derive(RuntimeConnectable)]
pub struct BinarySwitch<T> {
    #[output]
    pub output_1: Output<T>,
   
    #[output]
    pub output_2: Output<T>,
    
    #[input]
    pub input: Input<T>,

    switch: bool 
}

impl<T> BinarySwitch<T> {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output_1: Output::new(change_observer),
            output_2: Output::new(change_observer),
            input: Input::new(),
            switch : true
        }
    }
}

impl<T> Node for BinarySwitch<T> 
where T: Send {

    fn on_update(&mut self) -> Result<(), UpdateError> {
        if let Ok(input) = self.input.next() {
            self.switch = !self.switch;
            if self.switch {
                self.output_1.send(input).map_err(|e| UpdateError::Other(e.into()))?;
           } else {
                self.output_2.send(input).map_err(|e| UpdateError::Other(e.into()))?;
           }
        }
        Ok(())
    }
}

#[test]
fn test_switch() {
    
    let n: i32 = 1024;
    let numbers: Vec<i32> = (0..n).collect();

    let mut switch_node: BinarySwitch<i32> = BinarySwitch::new(None);
    
    let e_1: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();
    let e_2: flowrs::connection::Edge<i32> = flowrs::connection::Edge::new();

    connect(switch_node.output_1.clone(), e_1.clone());
    connect(switch_node.output_2.clone(), e_2.clone());
    
    for i in 0..n {
        let _ = switch_node.input.send(i);
        let _ = switch_node.on_update();
    }

    let odd_numbers: Vec<_> = numbers.iter().filter(|&x| x % 2 != 0).cloned().collect();
    let even_numbers: Vec<_> = numbers.iter().filter(|&x| x % 2 == 0).cloned().collect();

    let mut odd_res_nums = Vec::new(); 
    while let Ok(v) = e_1.next() {
        odd_res_nums.push(v);
    }

    let mut even_res_nums = Vec::new(); 
    while let Ok(v) = e_2.next() {
        even_res_nums.push(v);
    }

    //println!("{:?}", odd_res_nums);
    //println!("{:?}", even_res_nums);
    
    assert_eq!(odd_res_nums, odd_numbers);
    assert_eq!(even_res_nums, even_numbers);

}