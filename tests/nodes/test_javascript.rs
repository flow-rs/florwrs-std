#[cfg(test)]
mod nodes {
    use flowrs::{
        connection::{connect, Edge},
        node::{ChangeObserver, Node},
    };
    use flowrs_std::nodes::javascript::JsNode;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    struct TestArticle {
        quantity: usize,
        price: f32,
    }

    #[derive(Deserialize, Clone)]
    struct TestOutput {
        a: i32,
        b: f64,
    }

    #[test]
    fn transform_object() {
        let change_observer: ChangeObserver = ChangeObserver::new();
        let mut js_node = JsNode::new(Some(&change_observer));
        js_node
            .code_input
            .send(
                r#"
function main(input) {
    input.quantity += 5;

    return {
        a: input.quantity,
        b: Math.sqrt(input.price),
    };
}
"#
                .to_string(),
            )
            .unwrap();
        js_node
            .input
            .send(TestArticle {
                quantity: 2,
                price: 9.0,
            })
            .unwrap();
        let mock_output = Edge::<TestOutput>::new();
        connect(js_node.output.clone(), mock_output.clone());
        js_node.on_ready().unwrap();
        js_node.on_update().unwrap();

        let result = mock_output.next().unwrap();
        assert_eq!(result.a, 7);
        assert_eq!(result.b, 3.0)
    }

    #[test]
    fn load_module() {
        let change_observer: ChangeObserver = ChangeObserver::new();
        let mut js_node = JsNode::new(Some(&change_observer));
        js_node
            .code_input
            .send(
                r#"
function main() {
    let calc = require("./scripts/calc.js");
    return calc.six(); // FIXME: throws weird index out of bounds error if functions have parameters...
    // return calc.increment(5); // fails
    // return calc.add(3, 3); // fails
}
"#
                .to_string(),
            )
            .unwrap();
        js_node.input.send(()).unwrap();
        let mock_output = Edge::<i32>::new();
        connect(js_node.output.clone(), mock_output.clone());
        js_node.on_ready().unwrap();
        js_node.on_update().unwrap();

        let result = mock_output.next().unwrap();
        assert_eq!(result, 6);
    }
}
