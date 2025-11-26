use leptos::prelude::*;

use crate::components::result_card::{ResultCard, ResultData};

#[component]
pub fn ResultsStream(result_type: String) -> impl IntoView {
    let results = RwSignal::new(vec![]);

    // Effect: re-run whenever result_type changes
    Effect::new(move |_| {
        let _ = &result_type; // track dependency

        // Same simulated base results
        let base_results = vec![
            ResultData {
                id: 1,
                county: "Baringo".into(),
                leading: "David Samoei".into(),
                percentage: 52.0,
                opponent: "Mary Samoei".into(),
                opponent_percentage: 48.0,
                votes: "234,100".into(),
                polls_centers_reporting: "95%".into(),
            },
            ResultData {
                id: 2,
                county: "Kilifi".into(),
                leading: "John David".into(),
                percentage: 54.0,
                opponent: "Jane Doe".into(),
                opponent_percentage: 46.0,
                votes: "456,300".into(),
                polls_centers_reporting: "88%".into(),
            },
            ResultData {
                id: 3,
                county: "Embu".into(),
                leading: "Jane Doe".into(),
                percentage: 51.0,
                opponent: "John David".into(),
                opponent_percentage: 49.0,
                votes: "123,400".into(),
                polls_centers_reporting: "92%".into(),
            },
            ResultData {
                id: 4,
                county: "Kakamega".into(),
                leading: "Mark Smith".into(),
                percentage: 55.0,
                opponent: "Mary Smith".into(),
                opponent_percentage: 45.0,
                votes: "892,100".into(),
                polls_centers_reporting: "87%".into(),
            },
            ResultData {
                id: 5,
                county: "Samburu".into(),
                leading: "Nelly Nanyuki".into(),
                percentage: 50.5,
                opponent: "John David".into(),
                opponent_percentage: 49.5,
                votes: "456,800".into(),
                polls_centers_reporting: "84%".into(),
            },
            ResultData {
                id: 6,
                county: "Nyeri".into(),
                leading: "Jane Njeru".into(),
                percentage: 51.2,
                opponent: "John David".into(),
                opponent_percentage: 48.8,
                votes: "234,500".into(),
                polls_centers_reporting: "79%".into(),
            },
        ];

        results.set(base_results);
    });

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
