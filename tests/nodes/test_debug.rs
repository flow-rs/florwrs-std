#[cfg(test)]
mod nodes {
    
    
    use flowrs_std::debug::DebugNode;
    use flowrs::{connection::{Edge, connect}, node::{ChangeObserver, Node}};

    #[test]
    fn should_add_132() -> Result<(), anyhow::Error> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 

        let mock_output = Edge::new();
        let mut fst = DebugNode::new(Some(&change_observer));
        let mut snd = DebugNode::new(Some(&change_observer));
        connect(fst.output.clone(), snd.input.clone());
        connect(snd.output.clone(), mock_output.clone());
        let _ = fst.input.send(1);
        let _ = fst.on_update();
        let _ = snd.on_update();

        let expected = 1;
        let actual = mock_output.next()?;
        Ok(assert!(expected == actual))
    }
    
}
