//! Dashboard tab — 4 metric cards, member-growth line chart, fund bar chart
//! with the $2,500 cap line, and a one-click CSV download.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::analytics as an_api;
use crate::components::charts::{BarChart, LineChart};

#[component]
pub fn DashboardTab() -> impl IntoView {
    // ── Custom analytics filters ──────────────���───────────────────────
    let (filter_period, set_filter_period) = create_signal(String::new());
    let (filter_date_from, set_filter_date_from) = create_signal(String::new());
    let (filter_date_to, set_filter_date_to) = create_signal(String::new());
    let (filter_category, set_filter_category) = create_signal(String::new());
    let (filter_role, set_filter_role) = create_signal(String::new());
    let (filter_version, set_filter_version) = create_signal(0u32);

    let members = create_resource(|| (), |_| async move { an_api::members().await });
    let funds = create_resource(
        move || (filter_version.get(), filter_period.get(), filter_date_from.get(), filter_date_to.get(), filter_category.get(), filter_role.get()),
        move |(_v, period, df, dt, cat, role)| async move {
            let filter = an_api::AnalyticsFilter {
                period: if period.is_empty() { None } else { Some(period) },
                date_from: if df.is_empty() { None } else { Some(df) },
                date_to: if dt.is_empty() { None } else { Some(dt) },
                category: if cat.is_empty() { None } else { Some(cat) },
                role: if role.is_empty() { None } else { Some(role) },
            };
            an_api::fund_summary_filtered(&filter).await
        },
    );
    let approvals = create_resource(|| (), |_| async move { an_api::approval_cycles().await });
    let events = create_resource(|| (), |_| async move { an_api::events().await });
    let (status, set_status) = create_signal::<Option<String>>(None);

    let schedule_csv = move |_| {
        let p = filter_period.get();
        let df = filter_date_from.get();
        let dt = filter_date_to.get();
        let cat = filter_category.get();
        let rl = filter_role.get();
        spawn_local(async move {
            let req = an_api::ScheduleReportRequest {
                report_type: "fund".into(),
                format: "csv".into(),
                period: if p.is_empty() { None } else { Some(p) },
                date_from: if df.is_empty() { None } else { Some(df) },
                date_to: if dt.is_empty() { None } else { Some(dt) },
                category: if cat.is_empty() { None } else { Some(cat) },
                role: if rl.is_empty() { None } else { Some(rl) },
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

            // ── Custom filter panel ──────────────────────────────────────
            <div class="sv-card" style="margin-bottom:16px;">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Filters"</h3>
                <div style="display:grid;grid-template-columns:repeat(5,1fr) auto;gap:10px;align-items:end;">
                    <div>
                        <label class="sv-label">"Period"</label>
                        <input class="sv-input" placeholder="2026-04"
                            prop:value=move || filter_period.get()
                            on:input=move |ev| set_filter_period.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"From"</label>
                        <input class="sv-input" type="date"
                            prop:value=move || filter_date_from.get()
                            on:input=move |ev| set_filter_date_from.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"To"</label>
                        <input class="sv-input" type="date"
                            prop:value=move || filter_date_to.get()
                            on:input=move |ev| set_filter_date_to.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"Category"</label>
                        <input class="sv-input" placeholder="e.g. grants"
                            prop:value=move || filter_category.get()
                            on:input=move |ev| set_filter_category.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"Role"</label>
                        <select class="sv-input" on:change=move |ev| set_filter_role.set(event_target_value(&ev))>
                            <option value="">"All roles"</option>
                            <option value="administrator">"Administrator"</option>
                            <option value="content_curator">"Content Curator"</option>
                            <option value="reviewer">"Reviewer"</option>
                            <option value="finance_manager">"Finance Manager"</option>
                            <option value="store_manager">"Store Manager"</option>
                        </select>
                    </div>
                    <button class="sv-btn-primary" style="height:38px;"
                        on:click=move |_| set_filter_version.update(|v| *v += 1)>
                        "Apply"
                    </button>
                </div>
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
