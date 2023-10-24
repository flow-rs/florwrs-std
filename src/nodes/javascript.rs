use boa_engine::{Context, JsError, Source};
use flowrs::{
    connection::Output,
    node::{ChangeObserver, Node, UpdateError},
    RuntimeConnectable,
};
use serde::{Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct JsNode {
    #[output]
    pub output: Output<serde_json::Value>,
}

impl JsNode {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            output: Output::new(change_observer),
        }
    }
}

impl Node for JsNode {
    fn on_update(&mut self) -> anyhow::Result<(), UpdateError> {
        // TODO: make this dynamic
        let code = r#"
        function main() {
            return {
                method: "POST",
                url: "https://www.example.com",
                headers: {
                    "Content-Type": "application/json"
                }
            };
        }
        "#;
        let mut context = Context::default();
        let err_handler = |e: JsError| UpdateError::Other(anyhow::Error::msg(e.to_string()));
        context
            .eval(Source::from_bytes(code))
            .map_err(err_handler)?;

        let result = context
            .eval(Source::from_bytes("main()"))
            .map_err(err_handler)?;

        // scripts must return a value that can be converted into JSON
        let json = result.to_json(&mut context).map_err(err_handler)?;
        self.output.send(json)?;
        Ok(())
    }
}
