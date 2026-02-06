use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = Plotly)]
    pub fn newPlot(
        id: &str,
        data: &JsValue,
        layout: &JsValue,
    );
}
