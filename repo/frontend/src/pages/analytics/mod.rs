//! Analytics dashboard + scheduled reports pages.

use leptos::*;

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
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">"Analytics"</h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">"Member growth, fund summary, approval cycles, scheduled reports."</p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            <div style="display:flex;gap:8px;border-bottom:1px solid rgba(245,197,24,0.20);margin-bottom:24px;">
                {tab_btn("Dashboard", AnalyticsTab::Dashboard, tab, set_tab)}
                {tab_btn("Scheduled Reports", AnalyticsTab::Reports, tab, set_tab)}
            </div>

            {move || match tab.get() {
                AnalyticsTab::Dashboard => view! { <dashboard::DashboardTab /> }.into_view(),
                AnalyticsTab::Reports => view! { <reports::ReportsTab /> }.into_view(),
            }}
        </div>
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
