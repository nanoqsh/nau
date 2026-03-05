#![allow(async_fn_in_trait)]

use {
    async_channel::Receiver,
    futures_concurrency::stream::IntoStream,
    futures_lite::{Stream, stream},
    std::{marker::PhantomData, pin::Pin},
    wasm_bindgen::prelude::*,
    web_sys::{Document, Element, HtmlButtonElement},
};

pub async fn app<C>(comp: C, id: &str)
where
    C: Component,
{
    let document = web_sys::window().and_then(|w| w.document()).unwrap_throw();
    let root = document.get_element_by_id(id).unwrap_throw();
    let ui = Ui { document, root };
    comp.run_component(ui).await;
}

pub trait Html {
    fn append_child(&self, ui: &Ui);
}

struct OnEvent<A> {
    recv: Receiver<A>,
    _closure: Closure<dyn FnMut()>,
}

pub struct Button<A> {
    html: Option<HtmlButtonElement>,
    onclick: Option<OnEvent<A>>,
    action: PhantomData<fn(A)>,
}

impl<A> Button<A> {
    pub fn onclick<F>(mut self, mut f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        let (send, recv) = async_channel::unbounded();
        let closure = Closure::<dyn FnMut()>::new(move || _ = send.force_send(f()));

        let html = self.html.take().expect("html element");
        html.set_onclick(Some(closure.as_ref().unchecked_ref()));

        let onclick = OnEvent {
            recv,
            _closure: closure,
        };

        Self {
            html: Some(html),
            onclick: Some(onclick),
            action: PhantomData,
        }
    }

    pub async fn event(&self) -> Option<A> {
        self.onclick.as_ref()?.recv.recv().await.ok()
    }

    pub fn into_stream(self) -> impl Stream<Item = A> {
        stream::unfold(self, async |me| {
            let action = me.onclick.as_ref()?.recv.recv().await.ok()?;
            Some((action, me))
        })
    }
}

impl<A> Drop for Button<A> {
    fn drop(&mut self) {
        if let Some(html) = &self.html {
            html.remove();
        }
    }
}

impl<A> Html for Button<A> {
    fn append_child(&self, ui: &Ui) {
        _ = ui
            .root
            .append_child(self.html.as_ref().expect("html element"));
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

        Button {
            html: Some(html),
            onclick: None,
            action: PhantomData,
        }
    }

    pub fn push<H>(&self, html: &H)
    where
        H: Html,
    {
        html.append_child(self);
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
