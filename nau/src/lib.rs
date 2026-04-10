#![allow(async_fn_in_trait)]

use {
    async_channel::{Receiver, Sender},
    async_executor::{LocalExecutor, Task},
    futures_concurrency::stream::IntoStream,
    futures_lite::{Stream, StreamExt, stream},
    std::{
        collections::HashMap,
        future,
        pin::Pin,
        rc::Rc,
        task::{Context, Poll},
    },
    wasm_bindgen::prelude::*,
    web_sys::{
        Document, Element, Event, HtmlButtonElement, HtmlDivElement, HtmlInputElement, InputEvent,
        MouseEvent, PointerEvent,
    },
};

pub mod prelude {
    pub use {
        crate::{Component as _, Html as _, Ui},
        futures_concurrency::{self, prelude::*},
        futures_lite::{self, prelude::*},
    };
}

pub async fn app<C>(comp: C, id: &str) -> C::Output
where
    C: Component,
{
    let document = web_sys::window().and_then(|w| w.document()).unwrap_throw();
    let Some(root) = document.get_element_by_id(id) else {
        panic!("html element with id {id} not found");
    };

    let ex = Rc::default();
    let ui = Ui { document, root, ex };
    ui.ex.clone().run(comp.run_component(ui)).await
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
}

impl<H> Html for &H
where
    H: Html,
{
    fn get_element(&self) -> &Element {
        (**self).get_element()
    }
}

struct Events<A> {
    callbacks: HashMap<&'static str, Closure<dyn FnMut(Event)>>,
    send: Sender<A>,
    recv: Receiver<A>,
}

impl<A> Events<A> {
    fn new() -> Self {
        let (send, recv) = async_channel::unbounded();
        Self {
            callbacks: HashMap::new(),
            send,
            recv,
        }
    }

    fn set(&mut self, html: &Element, ty: &'static str, callback: Closure<dyn FnMut(Event)>) {
        html.add_event_listener_with_callback(ty, callback.as_ref().unchecked_ref())
            .unwrap_throw();

        if let Some(prev) = self.callbacks.insert(ty, callback) {
            html.remove_event_listener_with_callback(ty, prev.as_ref().unchecked_ref())
                .unwrap_throw();
        }
    }

    fn set_callback<F>(&mut self, html: &Element, ty: &'static str, mut f: F)
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        let send = self.send.clone();
        self.set(html, ty, Closure::new(move |_| _ = send.force_send(f())))
    }

    fn set_callback_with<F, E>(&mut self, html: &Element, ty: &'static str, mut f: F)
    where
        F: FnMut(E) -> A + 'static,
        A: 'static,
        E: JsCast,
    {
        let send = self.send.clone();
        self.set(
            html,
            ty,
            Closure::new(move |event: Event| {
                let event = event.dyn_into().unwrap_throw();
                _ = send.force_send(f(event))
            }),
        )
    }

    async fn event(&self) -> A {
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

pub struct Button<A> {
    html: RemoveOnDrop<HtmlButtonElement>,
    events: Events<A>,
}

impl<A> Button<A> {
    fn new(html: HtmlButtonElement) -> Self {
        Self {
            html: RemoveOnDrop(html),
            events: Events::new(),
        }
    }

    pub fn onclick<F>(mut self, f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        self.events.set_callback(self.html.get(), "click", f);
        self
    }

    pub fn onclick_with<F>(mut self, f: F) -> Self
    where
        F: FnMut(PointerEvent) -> A + 'static,
        A: 'static,
    {
        self.events.set_callback_with(self.html.get(), "click", f);
        self
    }

    pub fn onmouseenter<F>(mut self, f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        self.events.set_callback(self.html.get(), "mouseenter", f);
        self
    }

    pub fn onmouseenter_with<F>(mut self, f: F) -> Self
    where
        F: FnMut(MouseEvent) -> A + 'static,
        A: 'static,
    {
        self.events
            .set_callback_with(self.html.get(), "mouseenter", f);

        self
    }

    pub fn onmouseleave<F>(mut self, f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        self.events.set_callback(self.html.get(), "mouseleave", f);
        self
    }

    pub fn onmouseleave_with<F>(mut self, f: F) -> Self
    where
        F: FnMut(MouseEvent) -> A + 'static,
        A: 'static,
    {
        self.events
            .set_callback_with(self.html.get(), "mouseleave", f);

        self
    }

    pub fn onmousemove<F>(mut self, f: F) -> Self
    where
        F: FnMut() -> A + 'static,
        A: 'static,
    {
        self.events.set_callback(self.html.get(), "mousemove", f);
        self
    }

    pub fn onmousemove_with<F>(mut self, f: F) -> Self
    where
        F: FnMut(MouseEvent) -> A + 'static,
        A: 'static,
    {
        self.events
            .set_callback_with(self.html.get(), "mousemove", f);

        self
    }

    pub async fn event(&self) -> A {
        self.events.event().await
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
        Box::pin(self.events.into_stream().map(move |item| {
            _ = &self.html; // do not drop html
            item
        }))
    }
}

pub struct Input<A> {
    html: RemoveOnDrop<HtmlInputElement>,
    events: Events<A>,
}

impl<A> Input<A> {
    fn new(html: HtmlInputElement) -> Self {
        Self {
            html: RemoveOnDrop(html),
            events: Events::new(),
        }
    }

    pub fn oninput<F>(self, mut f: F) -> Self
    where
        F: FnMut(String) -> A + 'static,
        A: 'static,
    {
        self.oninput_with(move |event| f(event.data().unwrap_or_default()))
    }

    pub fn oninput_with<F>(mut self, f: F) -> Self
    where
        F: FnMut(InputEvent) -> A + 'static,
        A: 'static,
    {
        self.events.set_callback_with(self.html.get(), "input", f);
        self
    }

    pub async fn event(&self) -> A {
        self.events.event().await
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
        Box::pin(self.events.into_stream().map(move |item| {
            _ = &self.html; // do not drop html
            item
        }))
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
    fn create_element<H>(&self, tag: &str) -> H
    where
        H: JsCast,
    {
        self.document
            .create_element(tag)
            .unwrap_throw()
            .dyn_into()
            .unwrap_throw()
    }

    pub fn make_button<S, A>(&self, text: S) -> Button<A>
    where
        S: AsRef<str>,
    {
        let html: HtmlButtonElement = self.create_element("button");
        let text = text.as_ref();
        html.set_text_content(if text.is_empty() { None } else { Some(text) });
        _ = self.root.append_child(&html);
        Button::new(html)
    }

    pub fn make_input<S, A>(&self, placeholder: S) -> Input<A>
    where
        S: AsRef<str>,
    {
        let html: HtmlInputElement = self.create_element("input");
        html.set_type("text");
        html.set_placeholder(placeholder.as_ref());
        _ = self.root.append_child(&html);
        Input::new(html)
    }

    pub fn make_div(&self) -> Div {
        let html: HtmlDivElement = self.create_element("div");
        _ = self.root.append_child(&html);
        Div::new(html)
    }

    pub fn make<C>(&self, comp: C) -> ComponentHandle<C::Output>
    where
        C: Component + 'static,
    {
        let ui = Self {
            document: self.document.clone(),
            root: self.root.clone(),
            ex: self.ex.clone(),
        };

        let task = self.ex.spawn(comp.run_component(ui));
        ComponentHandle { task }
    }
}

impl Html for Ui {
    fn get_element(&self) -> &Element {
        &self.root
    }
}

pub struct ComponentHandle<R> {
    task: Task<R>,
}

impl<R> ComponentHandle<R> {
    pub fn detach(self) {
        self.task.detach();
    }
}

impl<R> Future for ComponentHandle<R> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.task).poll(cx)
    }
}

/// A trait for components.
pub trait Component: Sized {
    type Output;

    /// Runs the component with passed [ui](crate::Ui).
    async fn run_component(self, ui: Ui) -> Self::Output;

    /// Sets parent to the component.
    fn with_parent<H>(self, parent: H) -> WithParent<H, Self> {
        WithParent { parent, comp: self }
    }
}

impl<C, R> Component for C
where
    C: AsyncFnOnce(Ui) -> R,
{
    type Output = R;

    async fn run_component(self, ui: Ui) -> Self::Output {
        self(ui).await
    }
}

pub struct WithParent<H, C> {
    parent: H,
    comp: C,
}

impl<H, C> Component for WithParent<H, C>
where
    H: Html,
    C: Component,
{
    type Output = C::Output;

    async fn run_component(self, mut ui: Ui) -> Self::Output {
        ui.root = self.parent.get_element().clone();
        self.comp.run_component(ui).await
    }
}

/// Creates permanent component from a function.
pub fn permanent<C, R>(comp: C) -> impl Component<Output = ()>
where
    C: FnOnce(Ui) -> R,
{
    struct Permanent<C> {
        comp: C,
    }

    impl<C, R> Component for Permanent<C>
    where
        C: FnOnce(Ui) -> R,
    {
        type Output = ();

        async fn run_component(self, ui: Ui) -> Self::Output {
            let _elements = (self.comp)(ui);
            future::pending().await
        }
    }

    Permanent { comp }
}
