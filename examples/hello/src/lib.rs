use {nau::prelude::*, std::future, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(hello, "root").await;
}

async fn hello(ui: Ui) {
    ui.text("hello!"); // set text in root element
    future::pending().await // no IO
}
