#[cfg(test)]
mod nodes {
    use flowrs_std::debug::DebugNode;
    use flowrs::{connection::{Edge, connect}, node::{ChangeObserver, Node}};

    #[test]
    fn should_add_132() -> Result<(), anyhow::Error> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let mock_output = Edge::new();
        let fst = DebugNode::new("AddNodeI32", &change_observer);
        let snd = DebugNode::new("AddNodeI32", &change_observer);
        connect(fst.output.clone(), snd.input.clone());
        connect(snd.output.clone(), mock_output.clone());
        let _ = fst.input.send(1);
        let _ = fst.update();
        let _ = snd.update();

        let expected = 1;
        let actual = mock_output.next_elem()?;
        Ok(assert!(expected == actual))
    }
}
