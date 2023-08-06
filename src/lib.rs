mod nodes;

use wasm_bindgen::prelude::wasm_bindgen;

pub use self::nodes::add;
pub use self::nodes::value;
pub use self::nodes::debug;
pub use self::nodes::timer;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
