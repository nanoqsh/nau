use {
    futures_lite::stream,
    nau::{Html, prelude::*},
    wasm_bindgen::prelude::*,
};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(spawner, "root").await;
}

async fn spawner(ui: Ui) {
    let make = ui.make_button("Make").class("button").onclick(|| ());

    loop {
        make.event().await;
        ui.make(&ui, counter);
    }
}

async fn counter(ui: Ui) {
    let close = ui.make_button("Close").class("button").onclick(|| ());

    enum Event {
        Increment,
        Decrement,
    }

    let inc = ui
        .make_button("+1")
        .class("button")
        .onclick(|| Event::Increment);

    let text = ui.make_div().class("text").text("0");

    let dec = ui
        .make_button("-1")
        .class("button")
        .onclick(|| Event::Decrement);

    let _parent = ui.make_div().children(&[&close, &inc, &text, &dec]);

    let mut count = 0;
    stream::stop_after_future((inc, dec).merge(), close.event())
        .for_each(|event| {
            match event {
                Event::Increment => count += 1,
                Event::Decrement => count -= 1,
            }

            (&text).text(count.to_string());
        })
        .await;
}
