use leptos::prelude::*;

use crate::AppState;
use crate::components::election_map::ElectionMap;
use crate::components::result_table::ResultsTable;

#[component]
pub fn ResultsPage() -> impl IntoView {
    let result_type = use_context::<AppState>().map(|s| s.result_type).unwrap();
    view! {
        <>
            <div class="flex flex-1 overflow-hidden flex-col md:flex-row">

                <div class="w-full md:w-1/2 border-r-0 md:border-r border-border bg-card">
                    <ElectionMap result_type=result_type.get() />
                </div>
                <div class="hidden md:block w-full md:w-1/2 bg-background">
                    <ResultsTable result_type=result_type.get() />
                </div>
            </div>
        </>
    }
}
