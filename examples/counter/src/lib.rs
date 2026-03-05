use {gloo::console, nau::prelude::*, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(counter, "root").await;
}

async fn counter(ui: Ui) {
    enum Event {
        Increment,
        Decrement,
        Input(String),
    }

    let inc = ui
        .make_button("+1")
        .class("button")
        .onclick(|| Event::Increment);

    let input = ui
        .make_input("Input text..")
        .class("input")
        .oninput(Event::Input);

    let dec = ui
        .make_button("-1")
        .class("button")
        .onclick(|| Event::Decrement);

    let mut text = String::new();
    let mut count = 0;

    (inc, input, dec)
        .merge()
        .for_each(|event| match event {
            Event::Increment => {
                count += 1;
                console::log!(&text, count);
            }
            Event::Decrement => {
                count -= 1;
                console::log!(&text, count);
            }
            Event::Input(t) => text = t,
        })
        .await;
}
