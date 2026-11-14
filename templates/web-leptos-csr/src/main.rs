use leptos::prelude::*;

#[component]
fn App() -> impl IntoView {
    let (count, set_count) = signal(0_i32);
    view! {
        <main>
            <h1>"{{project-name}}"</h1>
            <p>"Leptos CSR starter"</p>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                {move || count.get().to_string()}
            </button>
        </main>
    }
}

fn main() {
    mount_to_body(|| view! { <App /> });
}
