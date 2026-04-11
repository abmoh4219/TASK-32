//! Analytics dashboard + scheduled reports pages.

use leptos::*;

use crate::components::layout::{NavTarget, PageShell};

pub mod dashboard;
pub mod reports;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AnalyticsTab {
    Dashboard,
    Reports,
}

#[component]
pub fn AnalyticsPage() -> impl IntoView {
    let (tab, set_tab) = create_signal(AnalyticsTab::Dashboard);

    view! {
        <PageShell
            active=NavTarget::Analytics
            title="Analytics"
            subtitle="Member growth, fund summary, approval cycles, scheduled reports"
        >
            <div class="sv-page-header">
                <h1 class="sv-page-title">"Analytics Dashboard"</h1>
                <p class="sv-page-subtitle">"Member growth, fund summary vs. $2,500 cap, approval cycles, and scheduled reports."</p>
            </div>

            <div class="sv-tabs">
                {tab_btn("Dashboard", AnalyticsTab::Dashboard, tab, set_tab)}
                {tab_btn("Scheduled Reports", AnalyticsTab::Reports, tab, set_tab)}
            </div>

            {move || match tab.get() {
                AnalyticsTab::Dashboard => view! { <dashboard::DashboardTab /> }.into_view(),
                AnalyticsTab::Reports => view! { <reports::ReportsTab /> }.into_view(),
            }}
        </PageShell>
    }
}

fn tab_btn(
    label: &'static str,
    this_tab: AnalyticsTab,
    current: ReadSignal<AnalyticsTab>,
    setter: WriteSignal<AnalyticsTab>,
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
