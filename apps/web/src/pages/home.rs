use leptos::prelude::*;

use crate::components::election_header::ElectionHeader;
use crate::components::election_map::ElectionMap;
use crate::components::result_stream::ResultsStream;

#[component]
pub fn HomePage() -> impl IntoView {
    let result_type = RwSignal::new("senate".into());

    view! {
        <div class="flex flex-col h-screen bg-[color:var(--background)]">

            <ElectionHeader
                result_type=result_type
                on_result_type_change=Callback::new(move |v: String| {
                    result_type.set(v);
                })
            />

            <div class="flex flex-1 overflow-hidden flex-col md:flex-row">

                {} <div class="w-full md:w-1/2 border-r-0 md:border-r border-border bg-card">
                    <ElectionMap result_type=result_type.get() />
                </div>
                {} <div class="hidden md:block w-full md:w-1/2 bg-background">
                    <ResultsStream result_type=result_type.get() />
                </div>
            </div>
        </div>
    }
}
