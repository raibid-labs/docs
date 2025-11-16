//! Web demo example
//!
//! WASM-compiled version for browser deployment.
//! Demonstrates how to build and deploy webatui applications to the web.
//!
//! ## Building for Web
//! ```bash
//! # Install wasm-pack if not already installed
//! cargo install wasm-pack
//!
//! # Build for web
//! wasm-pack build --target web --features web
//!
//! # Or use trunk for development
//! trunk serve --features web
//! ```
//!
//! ## Features
//! - WASM compilation
//! - Browser-based rendering
//! - Web event handling
//! - Same component code as terminal version

#[cfg(feature = "web")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "web")]
use yew::prelude::*;

#[cfg(feature = "web")]
#[derive(Clone, PartialEq)]
struct WebDemoState {
    counter: i32,
    items: Vec<String>,
    selected_tab: usize,
}

#[cfg(feature = "web")]
impl Default for WebDemoState {
    fn default() -> Self {
        Self {
            counter: 0,
            items: vec![
                "Item 1".to_string(),
                "Item 2".to_string(),
                "Item 3".to_string(),
            ],
            selected_tab: 0,
        }
    }
}

#[cfg(feature = "web")]
#[function_component(WebDemo)]
fn web_demo() -> Html {
    let state = use_state(WebDemoState::default);

    let increment = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.counter += 1;
            state.set(new_state);
        })
    };

    let decrement = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state.counter -= 1;
            state.set(new_state);
        })
    };

    let add_item = {
        let state = state.clone();
        Callback::from(move |_| {
            let mut new_state = (*state).clone();
            new_state
                .items
                .push(format!("Item {}", new_state.items.len() + 1));
            state.set(new_state);
        })
    };

    let select_tab = {
        let state = state.clone();
        Callback::from(move |tab: usize| {
            let mut new_state = (*state).clone();
            new_state.selected_tab = tab;
            state.set(new_state);
        })
    };

    html! {
        <div class="webatui-container">
            <header class="webatui-header">
                <h1>{ "WebATUI Demo - Web Version" }</h1>
                <p>{ "Same code, different platform!" }</p>
            </header>

            <nav class="webatui-tabs">
                <button
                    class={if state.selected_tab == 0 { "tab active" } else { "tab" }}
                    onclick={
                        let select_tab = select_tab.clone();
                        Callback::from(move |_| select_tab.emit(0))
                    }
                >
                    { "Counter" }
                </button>
                <button
                    class={if state.selected_tab == 1 { "tab active" } else { "tab" }}
                    onclick={
                        let select_tab = select_tab.clone();
                        Callback::from(move |_| select_tab.emit(1))
                    }
                >
                    { "List" }
                </button>
                <button
                    class={if state.selected_tab == 2 { "tab active" } else { "tab" }}
                    onclick={
                        let select_tab = select_tab.clone();
                        Callback::from(move |_| select_tab.emit(2))
                    }
                >
                    { "About" }
                </button>
            </nav>

            <main class="webatui-content">
                {
                    match state.selected_tab {
                        0 => html! {
                            <div class="tab-content">
                                <h2>{ "Counter Demo" }</h2>
                                <div class="counter-display">
                                    <span class="counter-value">{ state.counter }</span>
                                </div>
                                <div class="button-group">
                                    <button class="btn btn-primary" onclick={decrement}>
                                        { "Decrement" }
                                    </button>
                                    <button class="btn btn-primary" onclick={increment}>
                                        { "Increment" }
                                    </button>
                                </div>
                                <p class="hint">{ "Click the buttons to change the counter value" }</p>
                            </div>
                        },
                        1 => html! {
                            <div class="tab-content">
                                <h2>{ "List Demo" }</h2>
                                <ul class="item-list">
                                    {
                                        for state.items.iter().enumerate().map(|(i, item)| {
                                            html! {
                                                <li key={i} class="list-item">{ item }</li>
                                            }
                                        })
                                    }
                                </ul>
                                <button class="btn btn-success" onclick={add_item}>
                                    { "Add Item" }
                                </button>
                                <p class="hint">{ "Click to add items to the list" }</p>
                            </div>
                        },
                        2 => html! {
                            <div class="tab-content">
                                <h2>{ "About WebATUI" }</h2>
                                <div class="about-content">
                                    <p>{ "WebATUI is a framework that allows you to build terminal UIs that work in both:" }</p>
                                    <ul>
                                        <li>{ "Native terminals (using ratatui)" }</li>
                                        <li>{ "Web browsers (using WASM + Yew)" }</li>
                                    </ul>
                                    <p>{ "Write once, run everywhere!" }</p>
                                    <div class="features">
                                        <h3>{ "Key Features:" }</h3>
                                        <ul>
                                            <li>{ "Unified API for terminal and web" }</li>
                                            <li>{ "Component-based architecture" }</li>
                                            <li>{ "State management" }</li>
                                            <li>{ "Event handling" }</li>
                                            <li>{ "Responsive layouts" }</li>
                                        </ul>
                                    </div>
                                </div>
                            </div>
                        },
                        _ => html! { <div>{ "Unknown tab" }</div> }
                    }
                }
            </main>

            <footer class="webatui-footer">
                <p>{ "WebATUI Reference Implementation | Built with Rust + WASM" }</p>
            </footer>
        </div>
    }
}

#[cfg(feature = "web")]
#[wasm_bindgen(start)]
pub fn run_app() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();

    // Initialize logger
    wasm_logger::init(wasm_logger::Config::default());

    // Render the app
    yew::Renderer::<WebDemo>::new().render();
}

#[cfg(feature = "web")]
fn main() {
    // Entry point is handled by wasm_bindgen(start)
}

#[cfg(not(feature = "web"))]
fn main() {
    eprintln!(
        "This example requires the 'web' feature.\n\n\
         To build for web:\n\
         1. Install wasm-pack: cargo install wasm-pack\n\
         2. Build: wasm-pack build --target web --features web\n\n\
         Or use trunk for development:\n\
         1. Install trunk: cargo install trunk\n\
         2. Serve: trunk serve --features web"
    );
    std::process::exit(1);
}
