#![allow(async_fn_in_trait)]

use {
    async_channel::{Receiver, Sender},
    async_executor::LocalExecutor,
    futures_concurrency::stream::IntoStream,
    futures_lite::{Stream, stream},
    std::{future, marker::PhantomData, pin::Pin, rc::Rc},
    wasm_bindgen::prelude::*,
    web_sys::{Document, Element, HtmlButtonElement, HtmlDivElement, HtmlInputElement},
};

pub mod prelude {
    pub use {
        crate::{Component as _, Html as _, Ui},
        futures_concurrency::{self, prelude::*},
        futures_lite::{self, prelude::*},
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

    let ex = Rc::default();
    let ui = Ui { document, root, ex };
    ui.ex.clone().run(comp.run_component(ui)).await;
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

pub trait Html {
    fn get_element(&self) -> &Element;

    fn class(self, class: &str) -> Self
    where
        Self: Sized,
    {
        _ = self.get_element().class_list().add_1(class);
        self
    }

    fn text<S>(self, text: S) -> Self
    where
        S: AsRef<str>,
        Self: Sized,
    {
        let text = text.as_ref();
        let value = if text.is_empty() { None } else { Some(text) };
        self.get_element().set_text_content(value);
        self
    }

    fn child<H>(self, html: &H) -> Self
    where
        H: Html,
        Self: Sized,
    {
        _ = self.get_element().append_child(html.get_element());
        self
    }

    fn children(self, children: &[&dyn Html]) -> Self
    where
        Self: Sized,
    {
        for child in children.as_ref() {
            _ = self.get_element().append_child(child.get_element());
        }

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

pub struct Div(RemoveOnDrop<HtmlDivElement>);

impl Div {
    fn new(html: HtmlDivElement) -> Self {
        Self(RemoveOnDrop(html))
    }
}

impl Html for Div {
    fn get_element(&self) -> &Element {
        self.0.get()
    }
}

pub struct Ui {
    document: Document,
    root: Element,
    ex: Rc<LocalExecutor<'static>>,
}

impl Ui {
    pub fn make_button<S, A>(&self, text: S) -> Button<A>
    where
        S: AsRef<str>,
    {
        let html = self
            .document
            .create_element("button")
            .unwrap_throw()
            .dyn_into::<HtmlButtonElement>()
            .unwrap_throw();

        let text = text.as_ref();
        html.set_text_content(if text.is_empty() { None } else { Some(text) });

        _ = self.root.append_child(&html);
        Button::new(html)
    }

    pub fn make_input<S, A>(&self, placeholder: &str) -> Input<A>
    where
        S: AsRef<str>,
    {
        let html = self
            .document
            .create_element("input")
            .unwrap_throw()
            .dyn_into::<HtmlInputElement>()
            .unwrap_throw();

        html.set_type("text");
        html.set_placeholder(placeholder.as_ref());

        _ = self.root.append_child(&html);
        Input::new(html)
    }

    pub fn make_div(&self) -> Div {
        let html = self
            .document
            .create_element("div")
            .unwrap_throw()
            .dyn_into::<HtmlDivElement>()
            .unwrap_throw();

        _ = self.root.append_child(&html);
        Div::new(html)
    }

    pub fn make<C>(&self, comp: C) -> ComponentHandle
    where
        C: Component + 'static,
    {
        let ui = Self {
            document: self.document.clone(),
            root: self.root.clone(),
            ex: self.ex.clone(),
        };

        self.ex.spawn(comp.run_component(ui)).detach();
        ComponentHandle {}
    }
}

impl Html for Ui {
    fn get_element(&self) -> &Element {
        &self.root
    }
}

pub struct ComponentHandle {
    // TODO
}

pub trait Component: Sized {
    async fn run_component(self, ui: Ui);

    fn with_root<H>(self, root: H) -> WithRoot<H, Self> {
        WithRoot { root, comp: self }
    }
}

impl<C> Component for C
where
    C: AsyncFnOnce(Ui),
{
    async fn run_component(self, ui: Ui) {
        self(ui).await;
    }
}

pub struct WithRoot<H, C> {
    root: H,
    comp: C,
}

impl<H, C> Component for WithRoot<H, C>
where
    H: Html,
    C: Component,
{
    async fn run_component(self, mut ui: Ui) {
        ui.root = self.root.get_element().clone();
        self.comp.run_component(ui).await;
    }
}
