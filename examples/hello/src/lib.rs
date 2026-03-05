use {
    futures_concurrency::prelude::*, futures_lite::prelude::*, gloo::console, nau::Ui,
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(hello, "root").await;
}

async fn hello(ui: Ui) {
    let inc = ui.make_button("+1").onclick(|| 1);
    let dec = ui.make_button("-1").onclick(|| -1);
    let mut counter = 0;

    (inc, dec)
        .merge()
        .for_each(|d| {
            counter += d;
            console::log!("count: ", counter);
        })
        .await;
}
