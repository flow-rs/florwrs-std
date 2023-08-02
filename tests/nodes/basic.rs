#[cfg(test)]
mod nodes {
    use flowrs::{
        connection::{connect, ConnectError, Edge},
        node::{Context, Node, State},
    };
    use flowrs_std::basic::BasicNode;

    #[test]
    fn should_send_on_ready() -> Result<(), ConnectError<i32>> {
        let context = State::new(Context::new());
        let node = BasicNode::new("My Node", context, 42);
        let mock_output = Edge::new();
        connect(node.output.clone(), mock_output.clone());
        node.on_ready();

        let expected = 42;
        let actual = mock_output.next_elem()?;
        Ok(assert!(expected == actual))
    }
}
