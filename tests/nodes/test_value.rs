#[cfg(test)]
mod nodes {
   
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
        nodes::node::ReceiveError
    };
    use flowrs_std::value::ValueNode;

    #[test]
    fn should_send_on_ready() -> Result<(), ReceiveError> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 
        let node = ValueNode::new(42, Some(&change_observer));
        let mock_output = Edge::new();
        connect(node.output.clone(), mock_output.clone());
        _ = node.on_ready();

        let expected = 42;
        let actual = mock_output.next_elem()?;
        Ok(assert!(expected == actual))
    }
     
}
