use leptos::prelude::*;

#[derive(Clone, Debug)]
struct Candidate {
    id: u32,
    name: String,
    party: String,
    votes: u32,
    percentage: f64,
}

#[component]
pub fn ResultsTable(
    #[prop(default = "Presidential".to_string())] result_type: String,
    #[prop(default = "Nairobi County".to_string())] region: String,
) -> impl IntoView {
    let candidates = RwSignal::new(Vec::<Candidate>::new());
    let reporting = RwSignal::new(92.0);
    let total_votes = RwSignal::new(0);

    // Simulate loading data
    Effect::new(move |_| {
        let mock_candidates = vec![
            Candidate {
                id: 1,
                name: "Raila Odinga".to_string(),
                party: "ODM".to_string(),
                votes: 2_456_789,
                percentage: 48.5,
            },
            Candidate {
                id: 2,
                name: "William Ruto".to_string(),
                party: "UDA".to_string(),
                votes: 2_234_567,
                percentage: 44.2,
            },
            Candidate {
                id: 3,
                name: "George Wajackoyah".to_string(),
                party: "Roots Party".to_string(),
                votes: 234_890,
                percentage: 4.6,
            },
            Candidate {
                id: 4,
                name: "David Mwaure".to_string(),
                party: "Agano Party".to_string(),
                votes: 135_678,
                percentage: 2.7,
            },
        ];

        let total: u32 = mock_candidates.iter().map(|c| c.votes).sum();
        total_votes.set(total);
        candidates.set(mock_candidates);
    });

    view! {
        <div class="flex flex-col h-full bg-white rounded-lg shadow-lg">
            <div class="p-6 space-y-4">
                // Header
                <div class="mb-6">
                    <h2 class="text-xl font-bold text-foreground mb-1">{region}- {result_type}</h2>
                    <p class="text-sm text-gray-600">
                        "Updated every 30 seconds • "
                        <span class="font-semibold">
                            {move || format!("{:.1}%", reporting.get())}
                        </span> " of stations reporting"
                    </p>
                </div>

                // Scrollable Table
                <div class="flex-1 overflow-y-auto">
                    <table class="w-full">
                        <thead class="sticky top-0 bg-gray-100 border-b border-gray-200">
                            <tr>
                                <th class="px-6 py-3 text-left text-sm font-semibold text-gray-900">
                                    "Candidate"
                                </th>
                                <th class="px-6 py-3 text-left text-sm font-semibold text-gray-900">
                                    "Party"
                                </th>
                                <th class="px-6 py-3 text-right text-sm font-semibold text-gray-900">
                                    "Votes"
                                </th>
                                <th class="px-6 py-3 text-right text-sm font-semibold text-gray-900">
                                    "Percentage"
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || candidates.get()
                                key=|candidate| candidate.id
                                children=move |candidate| {
                                    let is_leading = candidates
                                        .get()
                                        .iter()
                                        .max_by(|a, b| {
                                            a.percentage.partial_cmp(&b.percentage).unwrap()
                                        })
                                        .map(|c| c.id == candidate.id)
                                        .unwrap_or(false);

                                    view! {
                                        <tr class="border-b border-gray-200 hover:bg-gray-50 transition-colors cursor-pointer">
                                            <td class="px-6 py-4 text-sm font-semibold text-gray-900">
                                                {candidate.name.clone()}
                                                {is_leading
                                                    .then(|| {
                                                        view! {
                                                            <span class="ml-2 text-xs text-green-600">
                                                                "👑 Leading"
                                                            </span>
                                                        }
                                                    })}
                                            </td>
                                            <td class="px-6 py-4 text-sm text-gray-700">
                                                {candidate.party.clone()}
                                            </td>
                                            <td class="px-6 py-4 text-sm text-right font-medium text-gray-900">
                                                {format!("{:}", candidate.votes)}
                                            </td>
                                            <td class="px-6 py-4 text-sm text-right">
                                                <span class=format!(
                                                    "inline-block px-3 py-1 rounded-full text-xs font-bold {}",
                                                    if is_leading {
                                                        "bg-green-100 text-green-800"
                                                    } else {
                                                        "bg-blue-100 text-blue-800"
                                                    },
                                                )>{format!("{:.1}%", candidate.percentage)}</span>
                                            </td>
                                        </tr>
                                    }
                                }
                            />
                        </tbody>
                    </table>
                </div>

                // Footer Info
                <div class="px-6 py-4 border-t border-gray-200 bg-gray-50">
                    <div class="flex justify-between items-center text-xs text-gray-600">
                        <span>{move || format!("{} candidates", candidates.get().len())}</span>
                        <span class="font-semibold">
                            "Total votes: " {move || format!("{:}", total_votes.get())}
                        </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
