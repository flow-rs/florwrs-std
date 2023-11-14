use boa_engine::{property::Attribute, Context, JsError, JsValue, Source};
use flowrs::{
    connection::{Input, Output},
    node::{ChangeObserver, Node, UpdateError},
    RuntimeConnectable,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(RuntimeConnectable, Deserialize, Serialize)]
pub struct JsNode<I, O> {
    #[input]
    pub input: Input<I>,
    #[input]
    pub code_input: Input<String>,
    #[output]
    pub output: Output<O>,

    code: String,
}

impl<I, O> JsNode<I, O> {
    pub fn new(change_observer: Option<&ChangeObserver>) -> Self {
        Self {
            input: Input::new(),
            code_input: Input::new(),
            output: Output::new(change_observer),
            code: String::new(),
        }
    }
}

impl<I, O> Node for JsNode<I, O>
where
    I: Send + Serialize,
    O: Send + DeserializeOwned,
{
    fn on_update(&mut self) -> anyhow::Result<(), UpdateError> {
        if let Ok(code) = self.code_input.next() {
            self.code = code;
        }
        let Ok(input_obj) = self.input.next() else {
            return Ok(());
        };

        let mut context = Context::default();
        let js_err_map = |e: JsError| UpdateError::Other(anyhow::Error::msg(e.to_string()));
        context
            .eval(Source::from_bytes(&self.code))
            .map_err(js_err_map)?;

        let serialized = serde_json::to_value(input_obj).map_err(anyhow::Error::from)?;
        let input = JsValue::from_json(&serialized, &mut context).map_err(js_err_map)?;
        context
            .register_global_property("__input", input, Attribute::all())
            .map_err(js_err_map)?;

        // scripts must define a main() function with optional input arg as entrypoint
        let result = context
            .eval(Source::from_bytes("main(__input)"))
            .map_err(js_err_map)?;

        // scripts must return a value that can be converted into JSON
        let json = result.to_json(&mut context).map_err(js_err_map)?;
        let deserialized = serde_json::from_value(json).map_err(anyhow::Error::from)?;
        self.output.send(deserialized)?;
        Ok(())
    }
}
