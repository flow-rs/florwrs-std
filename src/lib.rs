mod nodes;

use wasm_bindgen::prelude::wasm_bindgen;

pub use self::nodes::add;
pub use self::nodes::value;
pub use self::nodes::debug;
pub use self::nodes::timer;

pub use self::nodes::transform::json;
pub use self::nodes::transform::binary;
pub use self::nodes::control::switch;
pub use self::nodes::io::file;

// Required for debug node
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
