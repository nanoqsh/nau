#![allow(async_fn_in_trait)]

use {
    async_channel::{Receiver, Sender},
    futures_concurrency::stream::IntoStream,
    futures_lite::{Stream, stream},
    std::{future, marker::PhantomData, pin::Pin},
    wasm_bindgen::prelude::*,
    web_sys::{Document, Element, HtmlButtonElement, HtmlInputElement},
};

pub mod prelude {
    pub use {
        crate::{Component as _, Html as _, Ui},
        futures_concurrency::prelude::*,
        futures_lite::prelude::*,
    };
}

pub async fn app<C>(comp: C, id: &str)
where
    C: Component,
{
    let document = web_sys::window().and_then(|w| w.document()).unwrap_throw();
    let Some(root) = document.get_element_by_id(id) else {
        panic!("html element with id {id} not found");
    };

    let ui = Ui { document, root };
    comp.run_component(ui).await;
}

struct RemoveOnDrop<H>(H)
where
    H: AsRef<Element>;

impl<H> RemoveOnDrop<H>
where
    H: AsRef<Element>,
{
    fn get(&self) -> &H {
        &self.0
    }
}

impl<H> Drop for RemoveOnDrop<H>
where
    H: AsRef<Element>,
{
    fn drop(&mut self) {
        self.0.as_ref().remove();
    }
}

pub trait Html: Sized {
    fn get_element(&self) -> &Element;

    fn class(self, class: &str) -> Self {
        _ = self.get_element().class_list().add_1(class);
        self
    }
}

impl<H> Html for &H
where
    H: Html,
{
    fn get_element(&self) -> &Element {
        (**self).get_element()
    }
}

pub struct Button<A> {
    html: RemoveOnDrop<HtmlButtonElement>,
    onclick: Option<Closure<dyn FnMut()>>,
    send: Sender<A>,
    recv: Receiver<A>,
    action: PhantomData<fn(A)>,
}

impl<A> Button<A> {
    fn new(html: HtmlButtonElement) -> Self {
        let (send, recv) = async_channel::unbounded();
        Self {
            html: RemoveOnDrop(html),
            onclick: None,
            send,
            recv,
            action: PhantomData,
        }
    }

    pub fn onclick<F>(mut self, mut f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        let onclick = Closure::<dyn FnMut()>::new({
            let send = self.send.clone();
            move || _ = send.force_send(f())
        });

        self.html
            .get()
            .set_onclick(Some(onclick.as_ref().unchecked_ref()));

        self.onclick = Some(onclick);
        self
    }

    pub async fn event(&self) -> A {
        match self.recv.recv().await {
            Ok(action) => action,
            Err(_) => future::pending().await,
        }
    }

    fn into_stream(self) -> impl Stream<Item = A> {
        stream::unfold(self, async |me| {
            let action = me.recv.recv().await.ok()?;
            Some((action, me))
        })
    }
}

impl<A> Html for Button<A> {
    fn get_element(&self) -> &Element {
        self.html.get()
    }
}

impl<A> IntoStream for Button<A>
where
    A: 'static,
{
    type Item = A;
    type IntoStream = Pin<Box<dyn Stream<Item = Self::Item>>>;

    fn into_stream(self) -> Self::IntoStream {
        Box::pin(self.into_stream())
    }
}

pub struct Input<A> {
    html: RemoveOnDrop<HtmlInputElement>,
    oninput: Option<Closure<dyn FnMut()>>,
    send: Sender<A>,
    recv: Receiver<A>,
    action: PhantomData<fn(A)>,
}

impl<A> Input<A> {
    fn new(html: HtmlInputElement) -> Self {
        let (send, recv) = async_channel::unbounded();
        Self {
            html: RemoveOnDrop(html),
            oninput: None,
            send,
            recv,
            action: PhantomData,
        }
    }

    pub fn oninput<F>(mut self, mut f: F) -> Self
    where
        F: FnMut(String) -> A + 'static,
        A: 'static,
    {
        let oninput = Closure::<dyn FnMut()>::new({
            let html = self.html.get().clone();
            let send = self.send.clone();
            move || {
                let s = html.value();
                _ = send.force_send(f(s))
            }
        });

        self.html
            .get()
            .set_oninput(Some(oninput.as_ref().unchecked_ref()));

        self.oninput = Some(oninput);
        self
    }

    pub async fn event(&self) -> A {
        match self.recv.recv().await {
            Ok(action) => action,
            Err(_) => future::pending().await,
        }
    }

    fn into_stream(self) -> impl Stream<Item = A> {
        stream::unfold(self, async |me| {
            let action = me.recv.recv().await.ok()?;
            Some((action, me))
        })
    }
}

impl<A> Html for Input<A> {
    fn get_element(&self) -> &Element {
        self.html.get()
    }
}

impl<A> IntoStream for Input<A>
where
    A: 'static,
{
    type Item = A;
    type IntoStream = Pin<Box<dyn Stream<Item = Self::Item>>>;

    fn into_stream(self) -> Self::IntoStream {
        Box::pin(self.into_stream())
    }
}

pub struct Ui {
    document: Document,
    root: Element,
}

impl Ui {
    pub fn make_button<'text, S, A>(&self, text: S) -> Button<A>
    where
        S: Into<Option<&'text str>>,
    {
        let html = self
            .document
            .create_element("button")
            .unwrap_throw()
            .dyn_into::<HtmlButtonElement>()
            .unwrap_throw();

        html.set_text_content(text.into());
        _ = self.root.append_child(&html);

        Button::new(html)
    }

    pub fn make_input<A>(&self, placeholder: &str) -> Input<A> {
        let html = self
            .document
            .create_element("input")
            .unwrap_throw()
            .dyn_into::<HtmlInputElement>()
            .unwrap_throw();

        html.set_type("text");
        html.set_placeholder(placeholder);
        _ = self.root.append_child(&html);

        Input::new(html)
    }
}

pub trait Component: Sized {
    async fn run_component(self, ui: Ui);
}

impl<C> Component for C
where
    C: AsyncFnOnce(Ui),
{
    async fn run_component(self, ui: Ui) {
        self(ui).await;
    }
}
