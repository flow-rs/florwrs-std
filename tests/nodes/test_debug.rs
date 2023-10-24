#[cfg(test)]
mod nodes {

    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::debug::DebugNode;

    #[test]
    fn should_add_132() -> Result<(), anyhow::Error> {
        let change_observer: ChangeObserver = ChangeObserver::new();

        let mock_output = Edge::new();
        let mut fst = DebugNode::new(Some(&change_observer));
        let mut snd = DebugNode::new(Some(&change_observer));
        connect(fst.output.clone(), snd.input.clone());
        connect(snd.output.clone(), mock_output.clone());
        fst.input.send(1)?;
        fst.on_update()?;
        snd.on_update()?;

        let expected = 1;
        let actual = mock_output.next()?;
        Ok(assert!(expected == actual))
    }
}
