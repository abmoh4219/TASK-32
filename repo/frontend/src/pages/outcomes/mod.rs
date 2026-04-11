//! Outcome / IP Leptos pages — list, multi-step register, side-by-side compare.

use leptos::*;

pub mod register;
pub mod compare;

use crate::api::outcomes as out_api;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OutcomeTab {
    List,
    Register,
    Compare,
}

#[component]
pub fn OutcomesPage() -> impl IntoView {
    let (tab, set_tab) = create_signal(OutcomeTab::List);
    let outcomes = create_resource(|| (), |_| async move { out_api::list_outcomes().await });

    view! {
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">
                        "Outcome / IP Registration"
                    </h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">
                        "Register papers, patents, competition results, and software copyrights."
                    </p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            <div style="display:flex;gap:8px;border-bottom:1px solid rgba(245,197,24,0.20);margin-bottom:24px;">
                {tab_button("My Outcomes", OutcomeTab::List, tab, set_tab)}
                {tab_button("Register New", OutcomeTab::Register, tab, set_tab)}
                {tab_button("Compare", OutcomeTab::Compare, tab, set_tab)}
            </div>

            {move || match tab.get() {
                OutcomeTab::List => view! {
                    <div class="sv-card">
                        <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                            {move || outcomes.get().map(|res| match res {
                                Ok(rows) if rows.is_empty() => view! {
                                    <div style="text-align:center;color:#A0A0B0;padding:32px;">
                                        "No outcomes registered yet — switch to the Register tab to add one."
                                    </div>
                                }.into_view(),
                                Ok(rows) => view! {
                                    <table class="sv-table">
                                        <thead>
                                            <tr>
                                                <th>"Title"</th>
                                                <th>"Type"</th>
                                                <th>"Status"</th>
                                                <th>"Created"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows.into_iter().map(|o| view! {
                                                <tr>
                                                    <td>{o.title}</td>
                                                    <td><span class="sv-badge sv-badge-info">{o.r#type}</span></td>
                                                    <td>{render_status_badge(&o.status)}</td>
                                                    <td style="color:#A0A0B0;font-size:11px;">{o.created_at}</td>
                                                </tr>
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                }.into_view(),
                                Err(e) => view! { <div class="sv-error">{format!("Failed: {}", e.message)}</div> }.into_view(),
                            })}
                        </Suspense>
                    </div>
                }.into_view(),
                OutcomeTab::Register => view! { <register::RegisterOutcome /> }.into_view(),
                OutcomeTab::Compare => view! { <compare::CompareOutcomes /> }.into_view(),
            }}
        </div>
    }
}

fn render_status_badge(status: &str) -> View {
    let class = match status {
        "approved" => "sv-badge sv-badge-success",
        "rejected" => "sv-badge sv-badge-danger",
        "submitted" => "sv-badge sv-badge-warning",
        _ => "sv-badge sv-badge-info",
    };
    view! { <span class=class>{status.to_string()}</span> }.into_view()
}

fn tab_button(
    label: &'static str,
    this_tab: OutcomeTab,
    current: ReadSignal<OutcomeTab>,
    setter: WriteSignal<OutcomeTab>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| setter.set(this_tab)
            style=move || {
                let active = current.get() == this_tab;
                format!(
                    "padding:12px 20px;background:transparent;border:none;cursor:pointer;font-size:13px;font-weight:600;color:{};border-bottom:2px solid {};margin-bottom:-1px;",
                    if active { "#F5C518" } else { "#A0A0B0" },
                    if active { "#F5C518" } else { "transparent" }
                )
            }
        >
            {label}
        </button>
    }
}
