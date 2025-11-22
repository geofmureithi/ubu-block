use leptos::prelude::*;

#[component]
pub fn ElectionHeader(
    #[prop(into)] result_type: RwSignal<String>,
    on_result_type_change: Callback<String>,
) -> impl IntoView {
    
    let result_types = vec![
        ("presidential", "Presidential"),
        ("senate", "Senate"),
        ("governor", "Governor"),
        ("house", "MP"),
        ("local", "MCA"),
    ];

    view! {
        <div class="border-b border-border bg-card shadow-sm">
            <div class="px-4 md:px-6 py-4">
                <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">

                    // Left side
                    <div class="flex items-center gap-3">
                        <div class="w-2 h-8 bg-primary rounded-full bg-[color:var(--primary)]"></div>
                        <div>
                            <h1 class="text-xl md:text-2xl font-bold text-foreground">
                                "By-election Results 2025"
                            </h1>
                            <p class="text-xs md:text-sm text-muted-foreground">
                                "Live results powered by ubu-block"
                            </p>
                        </div>
                    </div>

                    // Filter buttons
                    <div class="flex gap-2 flex-wrap">
                        {result_types
                            .into_iter()
                            .map(|(id, label)| {
                                let is_selected = move || result_type.get() == id;
                                let variant_classes = move || {
                                    if is_selected() {
                                        "bg-primary text-primary-foreground hover:bg-primary/90"
                                    } else {
                                        "text-foreground border-border hover:bg-muted"
                                    }
                                };

                                view! {
                                    <button
                                        class=move || {
                                            format!(
                                                "px-3 py-1 rounded border text-sm {}",
                                                variant_classes(),
                                            )
                                        }
                                        on:click=move |_| {
                                            on_result_type_change.run(id.to_string());
                                        }
                                    >
                                        {label}
                                    </button>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                </div>
            </div>
        </div>
    }
}
