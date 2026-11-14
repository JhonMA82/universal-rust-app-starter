use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut count = use_signal(|| 0_i32);
    rsx! {
        main {
            h1 { "{{project-name}}" }
            p { "Dioxus web starter" }
            button { onclick: move |_| count += 1, "Count: {count}" }
        }
    }
}
