#[cfg(test)]
mod nodes {
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::nodes::javascript::JsNode;

    #[test]
    fn evaluate_json() {
        let change_observer: ChangeObserver = ChangeObserver::new();
        let mut js_node = JsNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(js_node.output.clone(), mock_output.clone());
        js_node.on_ready().unwrap();
        js_node.on_update().unwrap();

        mock_output.next().unwrap();
    }
}
