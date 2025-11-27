use leptos::prelude::*;

// use crate::AppState;
use crate::components::election_map::ElectionMap;
use crate::components::result_stream::ResultsStream;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <>
            <div class="flex flex-1 overflow-hidden flex-col md:flex-row">

                <div class="w-full md:w-1/2 border-r-0 md:border-r border-border bg-card">
                    <ElectionMap />
                </div>
                <div class="hidden md:block w-full md:w-1/2 bg-background">
                    <ResultsStream />
                </div>
            </div>
        </>
    }
}
