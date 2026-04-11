//! Outcome / IP Leptos pages — list, multi-step register, side-by-side compare.

use leptos::*;

pub mod register;
pub mod compare;

use crate::api::outcomes as out_api;
use crate::components::layout::{NavTarget, PageShell};

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
        <PageShell
            active=NavTarget::Outcomes
            title="Outcome / IP Registration"
            subtitle="Register papers, patents, competitions, and software copyrights"
        >
            <div class="sv-page-header">
                <h1 class="sv-page-title">"Outcome / IP Registration"</h1>
                <p class="sv-page-subtitle">"Register papers, patents, competition results, and software copyrights with full provenance."</p>
            </div>

            <div class="sv-tabs">
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
        </PageShell>
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
            class=move || if current.get() == this_tab { "sv-tab active" } else { "sv-tab" }
            on:click=move |_| setter.set(this_tab)
        >
            {label}
        </button>
    }
}
