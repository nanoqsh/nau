## nau

Experimental UI (web) framework

* Zero `Rc<RefCell<_>>` wrappers, `mut`-ate state directly
* Zero `use_state` hooks and etc
* Zero proc macros, components are *really* functions
* Rusty: use ownership to control components lifespans
* Event based, structured concurrency in mind
* Minimal API, reuse ready-made ecosystem

## Examples

An interactive component with a button and click counter:

```rust
use {nau::prelude::*, wasm_bindgen::prelude::*};

#[wasm_bindgen(start)]
pub async fn start() {
    nau::app(clicker, "root").await;
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
```
