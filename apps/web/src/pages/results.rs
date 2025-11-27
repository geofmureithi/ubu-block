use crate::components::result_table::ResultsTable;
use leptos::prelude::*;

use crate::AppState;
use crate::components::election_map::ElectionMap;

#[derive(Clone, Debug)]
struct FormData {
    result_type: String,
    county: String,
    station: String,
    constituency: String,
    ward: String,
}

impl Default for FormData {
    fn default() -> Self {
        Self {
            result_type: String::new(),
            county: String::new(),
            station: String::new(),
            constituency: String::new(),
            ward: String::new(),
        }
    }
}

#[component]
pub fn ResultsPage() -> impl IntoView {
    view! {
        <>
            <div class="flex flex-1 overflow-hidden flex-col md:flex-row">

                <div class="w-full md:w-1/2 border-r-0 md:border-r border-border bg-card">
                    <ElectionMap />
                </div>
                <div class="hidden md:block w-full md:w-1/2 bg-background">
                    <div class="p-6">
                        <h2 class="text-xl font-bold text-foreground mb-1">Submitted Results</h2>
                        <p class="text-sm text-gray-600">
                            "Updated every 30 seconds • "
                            // <span class="font-semibold">
                            //     {move || format!("{:.1}%", reporting.get())}
                            // </span> " of stations reporting"
                        </p>
                    </div>
                    <div class="p-6 space-y-4">
                        <ResultForm />
                    </div>
                </div>
            </div>
        </>
    }
}

#[component]
pub fn ResultForm() -> impl IntoView {
    let counties = LocalResource::new(move || crate::api::counties());
    let app_state = use_context::<AppState>().unwrap();

    let form_data = RwSignal::new(FormData::default());

    // Signals to track selected IDs for API calls
    let selected_county_id = RwSignal::new(String::new());
    let selected_constituency_id = RwSignal::new(String::new());
    let selected_ward_id = RwSignal::new(String::new());

    // Resources for cascading dropdowns
    let constituencies = LocalResource::new(move || {
        let county_id = selected_county_id.get();
        async move {
            if county_id.is_empty() {
                Ok(vec![])
            } else {
                crate::api::constituencies(&county_id).await
            }
        }
    });

    let wards = LocalResource::new(move || {
        let constituency_id = selected_constituency_id.get();
        async move {
            if constituency_id.is_empty() {
                Ok(vec![])
            } else {
                crate::api::wards(&constituency_id).await
            }
        }
    });

    let stations = LocalResource::new(move || {
        let ward_id = selected_ward_id.get();
        async move {
            if ward_id.is_empty() {
                Ok(vec![])
            } else {
                crate::api::stations(&ward_id).await
            }
        }
    });

    let results = LocalResource::new(move || {
        let ward_id = selected_ward_id.get();
        let constituency_id = selected_constituency_id.get();
        async move {
            match form_data.get().result_type.as_str() {
                "Mca" if !ward_id.is_empty() => crate::api::results("Mca", &ward_id).await,
                "Mp" if !constituency_id.is_empty() => {
                    crate::api::results("Mp", &constituency_id).await
                }
                _ => Ok(vec![]),
            }
        }
    });

    view! {
        <div class="mx-auto">
            <form class="space-y-6">
                <div class="grid lg:grid-cols-5 gap-2">
                    <div>
                        <label
                            for="resultType"
                            class="block text-sm font-semibold text-gray-900 mb-2"
                        >
                            "Result Type"
                        </label>
                        <select
                            id="resultType"
                            name="resultType"
                            prop:value=move || form_data.get().result_type
                            on:change=move |ev| {
                                form_data
                                    .update(|data| {
                                        data.result_type = event_target_value(&ev);
                                    });
                            }
                            required
                            class="w-full px-4 py-2 rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        >
                            <option value="">"Select a result type"</option>
                            {move ||app_state.positions.get()
                                .unwrap_or(Ok(vec![]))
                                .unwrap_or_default()
                                .into_iter()
                                .map(|type_name| {
                                    view! { <option value=type_name>{type_name.clone()}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>

                    // County
                    <div>
                        <label for="county" class="block text-sm font-semibold text-gray-900 mb-2">
                            "County"
                        </label>
                        <select
                            id="county"
                            name="county"
                            prop:value=move || form_data.get().county
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                form_data
                                    .update(|data| {
                                        data.county = value.clone();
                                        // Reset dependent fields
                                        data.constituency = String::new();
                                        data.ward = String::new();
                                        data.station = String::new();
                                    });
                                selected_county_id.set(value);
                                selected_constituency_id.set(String::new());
                                selected_ward_id.set(String::new());
                            }
                            required
                            class="w-full px-4 py-2 rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        >
                            <option value="">"Select a county"</option>
                            {move || counties.get()
                                .unwrap_or(Ok(vec![]))
                                .unwrap_or_default()
                                .into_iter()
                                .map(|county| {
                                    view! { <option value=county.county_code>{county.county_name.clone()}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>

                    // Constituency
                    <div>
                        <label for="constituency" class="block text-sm font-semibold text-gray-900 mb-2">
                            "Constituency"
                        </label>
                        <select
                            id="constituency"
                            name="constituency"
                            prop:value=move || form_data.get().constituency
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                form_data
                                    .update(|data| {
                                        data.constituency = value.clone();
                                        // Reset dependent fields
                                        data.ward = String::new();
                                        data.station = String::new();
                                    });
                                selected_constituency_id.set(value);
                                selected_ward_id.set(String::new());
                            }
                            required
                            disabled=move || selected_county_id.get().is_empty()
                            class="w-full px-4 py-2 rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed"
                        >
                            <option value="">"Select a constituency"</option>
                            {move || constituencies.get()
                                .unwrap_or(Ok(vec![]))
                                .unwrap_or_default()
                                .into_iter()
                                .map(|constituency| {
                                    view! { <option value=constituency.constituency_code>{constituency.constituency_name.clone()}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>

                    // Wards
                    <div>
                        <label for="ward" class="block text-sm font-semibold text-gray-900 mb-2">
                            "Ward"
                        </label>
                        <select
                            id="ward"
                            name="ward"
                            prop:value=move || form_data.get().ward
                            on:change=move |ev| {
                                let value = event_target_value(&ev);
                                form_data
                                    .update(|data| {
                                        data.ward = value.clone();
                                        // Reset dependent fields
                                        data.station = String::new();
                                    });
                                selected_ward_id.set(value);
                            }
                            required
                            disabled=move || selected_constituency_id.get().is_empty()
                            class="w-full px-4 py-2 rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed"
                        >
                            <option value="">"Select a ward"</option>
                            {move || wards.get()
                                .unwrap_or(Ok(vec![]))
                                .unwrap_or_default()
                                .into_iter()
                                .map(|ward| {
                                    view! { <option value=ward.ward_code>{ward.ward_name.clone()}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>

                    // Station
                    <div>
                        <label for="station" class="block text-sm font-semibold text-gray-900 mb-2">
                            "Voting Station"
                        </label>
                        <select
                            id="station"
                            name="station"
                            prop:value=move || form_data.get().station
                            on:change=move |ev| {
                                form_data
                                    .update(|data| {
                                        data.station = event_target_value(&ev);
                                    });
                            }
                            required
                            disabled=move || selected_ward_id.get().is_empty()
                            class="w-full px-4 py-2 rounded-lg border border-gray-300 bg-white text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent disabled:bg-gray-100 disabled:cursor-not-allowed"
                        >
                            <option value="">"Select a station"</option>
                            {move || stations.get()
                                .unwrap_or(Ok(vec![]))
                                .unwrap_or_default()
                                .into_iter()
                                .map(|station| {
                                    view! { <option value=station.id>{station.station_name.clone()}</option> }
                                })
                                .collect::<Vec<_>>()}
                        </select>
                    </div>

                </div>
            </form>

             {move || view! {<ResultsTable results=results.get().unwrap_or(Ok(vec![])).unwrap_or_default() />}}

        </div>
    }
}
