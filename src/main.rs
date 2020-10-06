use wasm_bindgen::{prelude::*, JsCast};
use moxie::{runtime::Runtime, state, Commit, Key};
use std::cell::RefCell;

type State<T> = (Commit<T>, Key<T>);

const ELEMENT_ID: &str = "app";

macro_rules! log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(input: &str);
}

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

fn runtime_run_once() {
    RUNTIME.with(|runtime| {
        runtime.borrow_mut().run_once(root);
    });
}

fn window() -> web_sys::Window {
    web_sys::window().expect("window")
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("document")
}

fn main() {
    log!("main");
    console_error_panic_hook::set_once();

    runtime_run_once();
    runtime_run_once();
}

#[topo::nested]
fn root() {
    log!("root");

    let state_node = state(|| Node {
        node_ws: web_sys::Node::from(document().get_element_by_id(ELEMENT_ID).expect("root element"))
    });

    let (first_run, first_run_key) = state(|| true);
    if *first_run {
        first_run_key.set(false);
        panel(|state_node| {
            text("Panel 1", state_node)
        }, state_node.clone());
    }
    panel(|state_node| {
        text("Panel 2", state_node);
    }, state_node);
}

#[topo::nested]
fn panel(children: impl FnOnce(State<Node>), state_parent_node: State<Node>) {
    log!("panel");

    let state_node = state(|| {
        let el_ws = document().create_element("div").expect("element");
        let node_ws = web_sys::Node::from(el_ws);
        (*state_parent_node.0).node_ws.append_child(&node_ws);
        Node { node_ws: web_sys::Node::from(node_ws) }
    });
    children(state_node);
}

#[topo::nested]
fn text(text: &str, state_parent_node: State<Node>) {  
    log!("text");

    let state_node = state(|| {
        let el_ws = document().create_element("span").expect("element");
        let node_ws = web_sys::Node::from(el_ws);

        let text_node_ws = document().create_text_node(&text).unchecked_into::<web_sys::Node>();

        node_ws.append_child(&text_node_ws);

        (*state_parent_node.0).node_ws.append_child(&node_ws);
        Node { node_ws }
    });
    log!("text: {}", text);
}

struct Node {
    node_ws: web_sys::Node,
}

impl Drop for Node {
    fn drop(&mut self) {
        let parent = self.node_ws.parent_node().expect("parent Node");
        parent.remove_child(&self.node_ws);
        log!("Node dropped");
    }
}
