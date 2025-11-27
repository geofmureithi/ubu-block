use leptos::prelude::*;

use crate::components::result_card::{ResultCard, ResultData};

#[component]
pub fn ResultsStream() -> impl IntoView {
    let results = LocalResource::new(move || async move { crate::api::live().await });

    view! {
        <div class="h-full overflow-y-auto">
            <div class="p-6 space-y-4">

                <div class="mb-6">
                    <h2 class="text-xl font-bold text-foreground mb-1">"Live Results"</h2>
                    <p class="text-sm text-muted-foreground">"Updated every 30 seconds"</p>
                </div>

                <div class="space-y-3">
                    <For
                        each=move || results.get()
                            .unwrap_or(Ok(vec![]))
                            .unwrap_or_default()
                            .into_iter().map(|s| ResultData { id: s.candidate1_id as i32, county: s.station_name, leading: s.candidate1_name, percentage: s.candidate1_percentage as f32, opponent: s.candidate2_name.unwrap_or_default(), opponent_percentage: s.candidate2_percentage.unwrap_or_default() as f32, votes: (s.candidate1_votes + s.candidate2_votes.unwrap_or_default()).to_string(), polls_centers_reporting: 0.to_string() })
                        key=|res| res.id
                        children=move |res: ResultData| {
                            view! { <ResultCard result=res /> }
                        }
                    />
                </div>

                {}
                <div class="pt-4 flex justify-center">
                    <button class="text-sm text-primary hover:text-primary/80 transition-colors">
                        "Load more results →"
                    </button>
                </div>
            </div>
        </div>
    }
}
