#[cfg(test)]
mod nodes {
    use anyhow::Error;
    use flowrs::connection::{connect, Edge, Input, Output, RuntimeConnectable};
    use flowrs::node::{ChangeObserver, Node};
    use flowrs_std::add::AddNode;
    use std::any::Any;
    use std::rc::Rc;
    use std::{thread, vec};


    #[test]
    fn should_add_132() -> Result<(), Error> {
        let change_observer = ChangeObserver::new(); 
        
        let mut add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        let mock_output = Edge::new();
        connect(add.output_1.clone(), mock_output.clone());
        let _ = add.input_1.send(1);
        let _ = add.input_2.send(2);
        let _ = add.on_update();
        let _ = add.on_update();

        let expected = 3;
        let actual = mock_output.next()?;
        Ok(assert!(expected == actual))
    }

    /// Scenario:
    ///
    /// [0, 1, ..., 100]
    ///         \
    ///          >-<Add>--[100, 100, ..., 100]
    ///         /
    /// [100, 99, ..., 0]
    #[test]
    fn should_add_multiple_132_sequentially() -> Result<(), Error> {
        let change_observer = ChangeObserver::new(); 

        let mut add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        let mock_output = Edge::new();
        connect(add.output_1.clone(), mock_output.clone());
        (0..100).for_each(|int| {
            let _ = add.input_1.send(int);
        });
        (0..101).rev().for_each(|int| {
            let _ = add.input_2.send(int);
        });
        (0..100).for_each(|_| {
            let _ = add.on_update();
        });
        let mut actual = vec![];
        for _ in 0..100 {
            let curr = mock_output.next()?;
            actual.push(curr)
        }
        let exected = vec![100; 100];
        Ok(assert!(
            exected == actual,
            "expected was: {:?} while actual was {:?}",
            exected,
            actual
        ))
    }

    #[test]
    fn should_add_multiple_132_parallel() -> Result<(), Error> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let mut add1 = AddNode::new("AddNodeI32", Some(&change_observer));
        let mut add2 = AddNode::new("AddNodeI32", Some(&change_observer));
        let mock_output = Edge::new();
        connect(add1.output_1.clone(), add2.input_1.clone());
        connect(add2.output_1.clone(), mock_output.clone());
        (0..100).for_each(|int| {
            let _ = add1.input_1.send(int);
        });
        (0..101).rev().for_each(|int| {
            let _ = add1.input_2.send(int);
        });
        (0..100).rev().for_each(|_| {
            let _ = add2.input_2.send(1);
        });

        let handle1 = thread::spawn(move || {
            (0..100).for_each(|_| {
                match add1.on_update() {
                    Ok(_) => (),
                    Err(e) => println!("{:?}", e),
                };
            });
        });
        let handle2 = thread::spawn(move || {
            (0..100).for_each(|_| {
                match add2.on_update() {
                    Ok(_) => (),
                    Err(e) => println!("{:?}", e),
                };
            });
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        let mut actual = vec![];
        for _ in 0..100 {
            let curr = mock_output.next();
            actual.push(curr)
        }
        Ok(assert!(!actual.is_empty()))
    }

    #[test]
    fn should_return_lhs_at_runtime() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        let input1: Rc<dyn Any> = add.input_at(0);
        let input1_downcasted = input1.downcast::<Input<i32>>();
        assert!(input1_downcasted.is_ok())
    }

    #[test]
    fn should_return_rhs_at_runtime() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        let input1: Rc<dyn Any> = add.input_at(1);
        let input1_downcasted = input1.downcast::<Input<i32>>();
        assert!(input1_downcasted.is_ok())
    }

    #[test]
    fn should_return_output_at_runtime() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        let input1: Rc<dyn Any> = add.output_at(0);
        let input1_downcasted = input1.downcast::<Output<i32>>();
        assert!(input1_downcasted.is_ok())
    }

    #[test]
    #[should_panic(expected = "Index 2 out of bounds for AddNode with input len 2.")]
    fn should_fail_on_index_out_of_bounds() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        add.input_at(2);
    }

    #[test]
    #[should_panic(expected = "Index 1 out of bounds for AddNode with output len 1.")]
    fn should_fail_on_output_out_of_bounds() {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let add: AddNode<i32, i32, i32> = AddNode::new("AddNodeI32", Some(&change_observer));
        add.output_at(1);
    }
    
}
