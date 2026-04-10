use {nau::prelude::*, std::any::Any, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(
        async |ui: Ui| {
            ui.make(nau::permanent(hello)).detach();
            ui.make(clicker).await
        },
        "root",
    )
    .await;
}

fn hello(ui: Ui) -> impl Any {
    ui.make_div().text("hello!")
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
