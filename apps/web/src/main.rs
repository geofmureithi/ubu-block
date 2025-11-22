use leptos::{logging::log, prelude::*};
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};


use crate::pages::home::HomePage;

mod pages;
mod components;

#[component]
pub fn App() -> impl IntoView {
    log!("Hello Log! (from App component)");

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // // Create a provide global state
    // let data = RwSignal::new(AppState::new());
    // provide_context(data);

    view! {
        <>
            <Title text="Ubu Blockchain" />
            <Meta name="description" content="Ubu Blockchain Election Results Application" />
            <HomePage />
        </>
    }
}

fn main() {
    mount_to_body(|| view! { <App /> })
}
