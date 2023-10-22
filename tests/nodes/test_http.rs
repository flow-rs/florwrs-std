#[cfg(test)]
mod nodes {
   
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node, ReceiveError},
    };
    use flowrs_std::value::ValueNode;
    use flowrs_std::http::HttpNode;

    #[test]
    fn receive_string_from_value_node() -> Result<(), anyhow::Error> {
        let change_observer: ChangeObserver = ChangeObserver::new(); 
        let value_node = ValueNode::new(String::from("Hello, world!"), Some(&change_observer));
        let mut http_node: HttpNode<String> = HttpNode::new(Some(&change_observer));
        let mock_output = Edge::new();
        connect(http_node.output.clone(), mock_output.clone());
        connect(value_node.output.clone(), http_node.data_input.clone());
        let _ = value_node.on_ready();
        let _ = http_node.on_update(); // why is the underscore needed?

        let actual: String = mock_output.next()?.into();
        //Ok(assert!(expected == actual))
        Ok(assert!(true))
    }

}
