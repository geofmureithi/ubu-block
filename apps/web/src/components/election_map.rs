use leptos::prelude::*;

use crate::components::kenyan_map::KenyanMap;

#[component]
pub fn ElectionMap(result_type: String) -> impl IntoView {
    // Equivalent to the useMemo constant in React
    let map_data = vec![
        MapState {
            county: "Baringo",
            leading: "David Samoei",
            votes: 234_100,
            color: "from-blue-500",
        },
        MapState {
            county: "Kilifi",
            leading: "John David",
            votes: 456_300,
            color: "from-red-500",
        },
        MapState {
            county: "Embu",
            leading: "Jane Doe",
            votes: 123_400,
            color: "from-blue-500",
        },
        MapState {
            county: "Kakamega",
            leading: "Mark Smith",
            votes: 892_100,
            color: "from-red-500",
        },
        MapState {
            county: "Samburu",
            leading: "Nelly Nanyuki",
            votes: 456_800,
            color: "from-blue-500",
        },
    ];

    // Helper to format the header text (capitalize first letter)
    let formatted_type = {
        let mut chars = result_type.chars();
        match chars.next() {
            Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
            None => result_type.clone(),
        }
    };

    view! {
        <div class="h-full w-full flex flex-col items-center justify-center p-8 bg-gradient-to-br from-card via-background to-card">
            <div class="w-full h-full flex flex-col items-center justify-center gap-8">

                <KenyanMap />

                // Top Results Summary
                <div class="grid grid-cols-2 gap-4 w-full">
                    {map_data
                        .iter()
                        .take(2)
                        .map(|state| {
                            let gradient = format!(
                                "w-3 h-3 rounded-full bg-gradient-to-r {} to-opacity-50",
                                state.color,
                            );
                            let vote_text = format!(
                                "{:.1}M votes",
                                state.votes as f64 / 1_000_000.0,
                            );

                            view! {
                                <div class="p-4 rounded-lg border border-border bg-card hover:shadow-md transition-shadow cursor-pointer">
                                    <div class="flex items-start justify-between">
                                        <div>
                                            <p class="text-xl font-bold text-foreground">
                                                {state.county}
                                            </p>
                                            <p class="text-sm text-muted-foreground mt-1">
                                                {state.leading}
                                            </p>
                                        </div>
                                        <div class=gradient></div>
                                    </div>
                                    <p class="text-xs text-muted-foreground mt-2">{vote_text}</p>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>

                // Footer text
                <div class="text-center mt-1">
                    <p class="text-sm text-muted-foreground">
                        {format!("Viewing {} Results", formatted_type)}
                    </p>
                    <p class="text-xs text-muted-foreground mt-1">
                        "Interactive map updates in real-time"
                    </p>
                </div>
            </div>
        </div>
    }
}

// Simple struct for the static map items
#[derive(Clone)]
struct MapState {
    county: &'static str,
    leading: &'static str,
    votes: u64,
    color: &'static str,
}
