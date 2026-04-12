//! Dashboard tab — 4 metric cards, member-growth line chart, fund bar chart
//! with the $2,500 cap line, and a one-click CSV download.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::analytics as an_api;
use crate::components::charts::{BarChart, LineChart};

#[component]
pub fn DashboardTab() -> impl IntoView {
    let members = create_resource(|| (), |_| async move { an_api::members().await });
    let funds = create_resource(|| (), |_| async move { an_api::fund_summary().await });
    let approvals = create_resource(|| (), |_| async move { an_api::approval_cycles().await });
    let events = create_resource(|| (), |_| async move { an_api::events().await });
    let (status, set_status) = create_signal::<Option<String>>(None);

    let schedule_csv = move |_| {
        spawn_local(async move {
            let req = an_api::ScheduleReportRequest {
                report_type: "fund".into(),
                format: "csv".into(),
                period: None,
                date_from: None,
                date_to: None,
                category: None,
            };
            match an_api::schedule_report(req).await {
                Ok(r) => set_status.set(Some(format!("Scheduled report {}", r.id))),
                Err(e) => set_status.set(Some(format!("Schedule failed: {}", e.message))),
            }
        });
    };

    view! {
        <div>
            <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:24px;">
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:100px;"></div> }>
                    {move || members.get().map(|res| match res {
                        Ok(m) => view! { <MetricCard label="Members" value=m.current_total.to_string() sub=format!("+{} new / -{} churned", m.new_members, m.churned)/> }.into_view(),
                        Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:100px;"></div> }>
                    {move || funds.get().map(|res| match res {
                        Ok(f) => {
                            let badge = if f.over_budget { "OVER" } else { "ok" };
                            view! { <MetricCard label="Fund net" value=format!("${:.2}", f.net) sub=format!("cap ${:.2} • {}", f.budget_cap, badge)/> }.into_view()
                        }
                        Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:100px;"></div> }>
                    {move || events.get().map(|res| match res {
                        Ok(e) => view! { <MetricCard label="Events" value=e.total_events.to_string() sub=format!("{} participants", e.total_participants)/> }.into_view(),
                        Err(err) => view! { <div class="sv-error">{err.message}</div> }.into_view(),
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:100px;"></div> }>
                    {move || approvals.get().map(|res| match res {
                        Ok(s) => view! { <MetricCard label="Approval avg" value=format!("{:.0} min", s.avg_minutes) sub=format!("median {:.0} • n={}", s.median_minutes, s.count)/> }.into_view(),
                        Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:24px;">
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:200px;"></div> }>
                    {move || members.get().map(|res| match res {
                        Ok(m) => {
                            let mut series: Vec<_> = m.series.into_iter().rev().map(|s| (s.snapshot_date, s.total_members as f64)).collect();
                            if series.is_empty() { series.push(("—".into(), 0.0)); }
                            view! { <LineChart title="Member Growth".to_string() points=series/> }.into_view()
                        }
                        Err(_) => ().into_view(),
                    })}
                </Suspense>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:200px;"></div> }>
                    {move || funds.get().map(|res| match res {
                        Ok(f) => {
                            let bars = vec![
                                ("Income".to_string(), f.total_income),
                                ("Expense".to_string(), f.total_expense),
                            ];
                            view! { <BarChart title="Fund vs Budget".to_string() bars=bars cap=f.budget_cap/> }.into_view()
                        }
                        Err(_) => ().into_view(),
                    })}
                </Suspense>
            </div>

            <div class="sv-card">
                <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:12px;">
                    <h3 style="margin:0;font-size:14px;color:#F5C518;">"Exports"</h3>
                </div>
                <div style="display:flex;gap:10px;">
                    <a class="sv-btn-secondary"
                       href="javascript:void(0)"
                       on:click=schedule_csv>
                        "Schedule fund CSV"
                    </a>
                    <span style="color:#A0A0B0;font-size:11px;align-self:center;">
                        {move || status.get().unwrap_or_else(|| "Use Scheduled Reports tab to download".into())}
                    </span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MetricCard(
    #[prop(into)] label: String,
    #[prop(into)] value: String,
    #[prop(into)] sub: String,
) -> impl IntoView {
    view! {
        <div class="sv-card">
            <div style="font-size:11px;color:#A0A0B0;text-transform:uppercase;letter-spacing:0.05em;">{label}</div>
            <div class="sv-text-gradient" style="font-size:28px;font-weight:800;margin-top:6px;">{value}</div>
            <div style="font-size:11px;color:#A0A0B0;margin-top:4px;">{sub}</div>
        </div>
    }
}
