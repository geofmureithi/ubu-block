use leptos::{logging::log, prelude::*};
use leptos_meta::*;
use leptos_router::components::{Outlet, ParentRoute, Route, Router, Routes};

use leptos_router::path;

use crate::components::election_header::ElectionHeader;
use crate::pages::home::HomePage;
use crate::pages::results::ResultsPage;
use crate::pages::submit::SubmissionPage;

mod api;
mod components;
mod pages;

#[derive(Clone, Copy)]
pub struct AppState {
    pub result_type: RwSignal<String>,
    pub positions: LocalResource<Result<Vec<String>, String>>,
}

impl AppState {
    pub fn new(result_type: RwSignal<String>) -> Self {
        Self {
            result_type,
            positions: LocalResource::new(|| crate::api::positions()),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    log!("Hello Log! (from App component)");

    let result_type = RwSignal::new("senate".into());

    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    // Create a provide global state
    let data = AppState::new(result_type);
    provide_context(data);

    view! {
        <>
            <Title text="Ubu Blockchain" />
            <Meta name="description" content="Ubu Blockchain Election Results Application" />
            <div class="flex flex-col h-screen bg-[color:var(--background)]">
                <Router>
                    <Routes fallback=|| "Not found.">
                        <ParentRoute
                            path=path!("")
                            view=move || {
                                view! {
                                    <>
                                        <ElectionHeader
                                            result_type=result_type
                                            on_result_type_change=Callback::new(move |v: String| {
                                                result_type.set(v);
                                            })
                                        />
                                        <Outlet />
                                    </>
                                }
                            }
                        >

                            <Route path=path!("") view=HomePage />

                            <Route path=path!("results") view=ResultsPage />

                            <Route path=path!("submit") view=SubmissionPage />
                        </ParentRoute>
                    </Routes>
                </Router>
            </div>
        </>
    }
}

fn main() {
    mount_to_body(|| view! { <App /> })
}
