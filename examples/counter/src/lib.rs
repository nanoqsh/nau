use {
    gloo::timers::future::sleep,
    nau::{Component, Html, prelude::*},
    std::time::Duration,
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

        let parent = ui.make_div().class("counter");
        ui.make(counter.with_parent(parent));
    }
}

async fn counter(ui: Ui) {
    let close = ui.make_button("Close").class("button").onclick(|| ());
    let inc = ui.make_button("+1").class("button").onclick(|| 1);
    let text = ui.make_div().class("text").text("0");
    let dec = ui.make_button("-1").class("button").onclick(|| -1);

    let fadeout = async {
        close.event().await;
        (&ui).class("hide");

        // sleep before exit to play animation
        sleep(Duration::from_millis(500)).await;
    };

    let mut count = 0;
    let input = (inc, dec).merge().for_each(|d| {
        count += d;
        (&text).text(count.to_string());
    });

    (input, fadeout).race().await;
}
