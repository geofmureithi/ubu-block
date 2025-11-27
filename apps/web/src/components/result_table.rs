use leptos::prelude::*;
use types::results::GeneralResult;

#[component]
pub fn ResultsTable(results: Vec<GeneralResult>) -> impl IntoView {
    view! {
        <div class="flex flex-col h-full bg-white rounded-lg shadow-lg mt-6">
            <div class="p-6 space-y-4">
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
                                    "Standard Deviation"
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <For
                                each=move || results.clone()
                                key=|candidate| candidate.candidate_id
                                children=move |candidate| {
                                    // let is_leading = candidates
                                    //     .get()
                                    //     .iter()
                                    //     .max_by(|a, b| {
                                    //         a.percentage.partial_cmp(&b.percentage).unwrap()
                                    //     })
                                    //     .map(|c| c.id == candidate.id)
                                    //     .unwrap_or(false);
                                    let is_leading = false;

                                    view! {
                                        <tr class="border-b border-gray-200 hover:bg-gray-50 transition-colors cursor-pointer">
                                            <td class="px-6 py-4 text-sm font-semibold text-gray-900">
                                                {candidate.candidate_name.clone()}
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
                                                {candidate.party_title.clone()}
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
                                                )>{format!("{:.1}%", candidate.sd)}</span>
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
                        // <span>{move || format!("{} candidates", candidates.get().len())}</span>
                        // <span class="font-semibold">
                        //     "Total votes: " {move || format!("{:}", total_votes.get())}
                        // </span>
                    </div>
                </div>
            </div>
        </div>
    }
}
