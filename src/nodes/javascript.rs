use boa_engine::{
    property::Attribute, Context, JsArgs, JsError, JsNativeError, JsObject, JsResult, JsValue,
    NativeFunction, Source,
};
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

        let js_err_map = |e: JsError| UpdateError::Other(anyhow::Error::msg(e.to_string()));
        let mut context = prepare_context().map_err(js_err_map)?;

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

fn prepare_context<'a>() -> anyhow::Result<Context<'a>, JsError> {
    let mut context = Context::default();
    // register the "require" function
    context.register_global_callable("require", 0, NativeFunction::from_fn_ptr(require))?;
    // Adding custom object that mimics 'module.exports'
    let moduleobj = JsObject::default();
    moduleobj.set("exports", JsValue::Undefined, false, &mut context)?;
    context.register_global_property("module", JsValue::from(moduleobj), Attribute::default())?;
    Ok(context)
}

// FROM: https://github.com/boa-dev/boa/blob/main/boa_examples/src/bin/modulehandler.rs
fn require(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let arg = args.get_or_undefined(0);

    let libfile = arg.to_string(ctx)?.to_std_string_escaped();

    // Read the module source file
    let buffer = std::fs::read_to_string(libfile)
        .map_err(|e| JsNativeError::typ().with_message(e.to_string()))?;
    // Load and parse the module source
    // println!("{buffer}");
    ctx.eval(Source::from_bytes(&buffer))?;

    // Access module.exports and return as ResultValue
    let global_obj = ctx.global_object();
    let module = global_obj.get("module", ctx)?;
    module
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("`exports` property was not an object"))?
        .get("exports", ctx)
}
