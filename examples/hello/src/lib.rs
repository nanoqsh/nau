use {nau::prelude::*, std::future, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(
        async |ui: Ui| {
            let _hello = ui.make(hello);
            ui.make(clicker).await
        },
        "root",
    )
    .await;
}

async fn hello(ui: Ui) {
    ui.text("hello!"); // set text in root element
    future::pending().await // no IO
}

async fn clicker(ui: Ui) {
    let button = ui.make_button("Click").onclick(|| ());
    let text = ui.make_div();
    let mut n = 0;
    loop {
        (&text).text(format!("clicked {n} times.."));
        button.event().await;
        n += 1;
    }
}
