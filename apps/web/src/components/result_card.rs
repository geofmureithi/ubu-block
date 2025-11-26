use leptos::prelude::*;
use leptos_router::components::A;

#[derive(Clone)]
pub struct ResultData {
    pub id: i32,
    pub county: String,
    pub leading: String,
    pub percentage: f32,
    pub opponent: String,
    pub opponent_percentage: f32,
    pub votes: String,
    pub polls_centers_reporting: String,
}

#[component]
pub fn ResultCard(result: ResultData) -> impl IntoView {
    let leading_color = if result.percentage > result.opponent_percentage {
        "text-chart-1"
    } else {
        "text-chart-2"
    };

    view! {
        <div class="p-4 bg-card border border-border rounded-lg hover:shadow-md transition-shadow cursor-pointer group">
            <div class="flex items-center justify-between mb-3">
                <A href="/results">
                <h3 class="font-semibold text-foreground group-hover:text-primary transition-colors">
                    {result.county}
                </h3>
                </A>

                <span class="text-xs font-medium px-2.5 py-1 rounded-full bg-[color:var(--secondary)] text-[color:var(--secondary-foreground)]">
                    {result.polls_centers_reporting}
                </span>
            </div>

            <div class="space-y-2">
                <div>
                    <div class="flex items-center justify-between mb-1">
                        <span class=format!(
                            "font-semibold text-sm {}",
                            leading_color,
                        )>{result.leading}</span>

                        <span class="font-bold text-foreground">
                            {format!("{}%", result.percentage)}
                        </span>
                    </div>

                    <div class="w-full h-2 bg-[color:var(--muted)] rounded-full overflow-hidden">
                        <div
                            class={if result.percentage > result.opponent_percentage {
                                "h-full bg-[color:var(--chart-1)]"
                            } else {
                                "h-full bg-[color:var(--muted)]"
                            }}
                            style=format!("width: {}%;", result.percentage)
                        ></div>
                    </div>
                </div>
                <div>
                    <div class="flex items-center justify-between mb-1">
                        <span class="text-sm text-muted-foreground">{result.opponent}</span>

                        <span class="font-bold text-foreground">
                            {format!("{}%", result.opponent_percentage)}
                        </span>
                    </div>

                    <div class="w-full h-2 bg-[color:var(--muted)] rounded-full overflow-hidden">
                        <div
                            class={if result.opponent_percentage > result.percentage {
                                "h-full bg-chart-2"
                            } else {
                                "h-full bg-muted"
                            }}
                            style=format!("width: {}%;", result.opponent_percentage)
                        ></div>
                    </div>
                </div>
            </div>
            <div class="flex items-center justify-between mt-3 pt-3 border-t border-border">
                <span class="text-xs text-muted-foreground">
                    {format!("{} votes", result.votes)}
                </span>

                <span class="text-xs font-medium text-primary">"Live"</span>
            </div>
        </div>
    }
}
