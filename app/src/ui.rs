use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

pub fn launch_app() {
    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(WindowBuilder::new().with_resizable(true)))
        .launch(app)
}

fn app() -> Element {
    let handle_load_click = move |_| async {
        // Load embeddings logic...
    };

    let handle_lance_query_click = move |_| async {
        // LanceQuery logic...
    };

    let handle_rag_query_click = move |_| async {
        // RagQuery logic...
    };

    let handle_generate_click = move |_| async {
        // Generate AI response logic...
    };

    rsx! {
        button { onclick: handle_load_click, "Load" }
        button { onclick: handle_lance_query_click, "Lance Query" }
        button { onclick: handle_rag_query_click, "Rag Query" }
        button { onclick: handle_generate_click, "Generate" }
    }
}
