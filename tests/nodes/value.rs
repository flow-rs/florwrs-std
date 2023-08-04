#[cfg(test)]
mod nodes {
    use flowrs::{
        connection::{connect, ConnectError, Edge},
        node::{ChangeObserver, Node, State},
    };
    use flowrs_std::value::ValueNode;

    #[test]
    fn should_send_on_ready() -> Result<(), ConnectError<i32>> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 
        let node = ValueNode::new("My Node", &change_observer, 42);
        let mock_output = Edge::new();
        connect(node.output.clone(), mock_output.clone());
        node.on_ready();

        let expected = 42;
        let actual = mock_output.next_elem()?;
        Ok(assert!(expected == actual))
    }
}
