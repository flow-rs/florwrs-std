#[cfg(test)]
mod nodes {
    use flowrs_std::debug::DebugNode;
    use flowrs::{connection::{ConnectError, Edge, connect}, node::{ChangeObserver, State, Node}};

    #[test]
    fn should_add_132() -> Result<(), ConnectError<i32>> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let mock_output = Edge::new();
        let fst = DebugNode::new("AddNodeI32", &change_observer);
        let snd = DebugNode::new("AddNodeI32", &change_observer);
        connect(fst.output.clone(), snd.input.clone());
        connect(snd.output.clone(), mock_output.clone());
        let _ = fst.input.send(1);
        let _ = fst.on_update();
        let _ = snd.on_update();

        let expected = 1;
        let actual = mock_output.next_elem()?;
        Ok(assert!(expected == actual))
    }
}
