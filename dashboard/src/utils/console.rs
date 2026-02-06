use serde_wasm_bindgen::to_value;
use web_sys::console;

pub fn console_error(text: &String) {
    console::error_1(&to_value(text).expect("Cannot print console error messages"));
}

pub fn console_error_with(text: &'static str, value: String) {
    console::error_1(
        &to_value(&format!("{}: {}", text, value)).expect("Cannot print console error messages"),
    );
}
