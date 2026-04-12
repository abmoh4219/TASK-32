//! Scheduled reports tab — list pending/complete reports, schedule new ones,
//! download via the single-use token URL.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::analytics as an_api;

#[component]
pub fn ReportsTab() -> impl IntoView {
    let reports = create_resource(|| (), |_| async move { an_api::list_reports().await });
    let (status, set_status) = create_signal::<Option<String>>(None);
    let (report_type, set_report_type) = create_signal("fund".to_string());
    let (format_kind, set_format_kind) = create_signal("csv".to_string());
    let (period, set_period) = create_signal(String::new());
    let (date_from, set_date_from) = create_signal(String::new());
    let (date_to, set_date_to) = create_signal(String::new());
    let (category, set_category) = create_signal(String::new());
    let (role, set_role) = create_signal(String::new());

    let schedule = move |_| {
        let p = period.get();
        let df = date_from.get();
        let dt = date_to.get();
        let cat = category.get();
        let rl = role.get();
        let req = an_api::ScheduleReportRequest {
            report_type: report_type.get(),
            format: format_kind.get(),
            period: if p.is_empty() { None } else { Some(p) },
            date_from: if df.is_empty() { None } else { Some(df) },
            date_to: if dt.is_empty() { None } else { Some(dt) },
            category: if cat.is_empty() { None } else { Some(cat) },
            role: if rl.is_empty() { None } else { Some(rl) },
        };
        spawn_local(async move {
            match an_api::schedule_report(req).await {
                Ok(_) => {
                    set_status.set(Some("Scheduled".into()));
                    reports.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1fr 320px;gap:24px;">
            <div class="sv-card">
                <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"My Reports"</h2>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                    {move || reports.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div style="text-align:center;color:#A0A0B0;padding:24px;">"No reports yet."</div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead><tr><th>"Type"</th><th>"Status"</th><th>"Created"</th><th>"Action"</th></tr></thead>
                                <tbody>
                                    {rows.into_iter().map(|r| {
                                        let badge = match r.status.as_str() {
                                            "complete" => "sv-badge sv-badge-success",
                                            "pending" => "sv-badge sv-badge-warning",
                                            _ => "sv-badge sv-badge-info",
                                        };
                                        let download_url = match (r.download_token.as_ref(), r.status == "complete") {
                                            (Some(tok), true) => Some(format!("/api/analytics/reports/{}/download/{}", r.id, tok)),
                                            _ => None,
                                        };
                                        view! {
                                            <tr>
                                                <td>{r.report_type.clone()}</td>
                                                <td><span class=badge>{r.status.clone()}</span></td>
                                                <td style="color:#A0A0B0;font-size:11px;">{r.created_at.clone()}</td>
                                                <td>
                                                    {download_url.map(|u| view! {
                                                        <a class="sv-btn-secondary" href=u target="_blank" style="font-size:11px;padding:6px 12px;">"Download"</a>
                                                    })}
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div class="sv-card">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Schedule a Report"</h3>
                <label class="sv-label">"Type"</label>
                <select class="sv-input" on:change=move |ev| set_report_type.set(event_target_value(&ev))>
                    <option value="fund">"Fund summary"</option>
                    <option value="members">"Members snapshot"</option>
                    <option value="events">"Event participation"</option>
                </select>
                <label class="sv-label" style="margin-top:10px;">"Format"</label>
                <select class="sv-input" on:change=move |ev| set_format_kind.set(event_target_value(&ev))>
                    <option value="csv">"CSV"</option>
                    <option value="pdf">"PDF"</option>
                </select>
                <label class="sv-label" style="margin-top:10px;">"Period (optional, YYYY-MM)"</label>
                <input
                    class="sv-input"
                    placeholder="2026-04"
                    prop:value=move || period.get()
                    on:input=move |ev| set_period.set(event_target_value(&ev))
                />
                <label class="sv-label" style="margin-top:10px;">"Date from"</label>
                <input class="sv-input" type="date"
                    prop:value=move || date_from.get()
                    on:input=move |ev| set_date_from.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Date to"</label>
                <input class="sv-input" type="date"
                    prop:value=move || date_to.get()
                    on:input=move |ev| set_date_to.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Category"</label>
                <input class="sv-input" placeholder="e.g. grants"
                    prop:value=move || category.get()
                    on:input=move |ev| set_category.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Role"</label>
                <select class="sv-input" on:change=move |ev| set_role.set(event_target_value(&ev))>
                    <option value="">"All roles"</option>
                    <option value="administrator">"Administrator"</option>
                    <option value="content_curator">"Content Curator"</option>
                    <option value="reviewer">"Reviewer"</option>
                    <option value="finance_manager">"Finance Manager"</option>
                    <option value="store_manager">"Store Manager"</option>
                </select>
                <button class="sv-btn-primary" style="margin-top:14px;width:100%;" on:click=schedule>
                    "Generate Report"
                </button>
                <div style="display:grid;grid-template-columns:1fr 1fr;gap:6px;margin-top:8px;">
                    <button
                        class="sv-btn-secondary"
                        style="text-align:center;font-size:11px;padding:8px;"
                        on:click=move |_| {
                            let p = period.get();
                            let df = date_from.get();
                            let dt = date_to.get();
                            let cat = category.get();
                            let rl = role.get();
                            let req = an_api::ScheduleReportRequest {
                                report_type: report_type.get(),
                                format: "csv".into(),
                                period: if p.is_empty() { None } else { Some(p) },
                                date_from: if df.is_empty() { None } else { Some(df) },
                                date_to: if dt.is_empty() { None } else { Some(dt) },
                                category: if cat.is_empty() { None } else { Some(cat) },
                                role: if rl.is_empty() { None } else { Some(rl) },
                            };
                            spawn_local(async move {
                                match an_api::schedule_report(req).await {
                                    Ok(_) => {
                                        set_status.set(Some("CSV export queued — refresh to download".into()));
                                        reports.refetch();
                                    }
                                    Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
                                }
                            });
                        }
                    >
                        "Export CSV"
                    </button>
                    <button
                        class="sv-btn-secondary"
                        style="text-align:center;font-size:11px;padding:8px;"
                        on:click=move |_| {
                            let p = period.get();
                            let df = date_from.get();
                            let dt = date_to.get();
                            let cat = category.get();
                            let rl = role.get();
                            let req = an_api::ScheduleReportRequest {
                                report_type: report_type.get(),
                                format: "pdf".into(),
                                period: if p.is_empty() { None } else { Some(p) },
                                date_from: if df.is_empty() { None } else { Some(df) },
                                date_to: if dt.is_empty() { None } else { Some(dt) },
                                category: if cat.is_empty() { None } else { Some(cat) },
                                role: if rl.is_empty() { None } else { Some(rl) },
                            };
                            spawn_local(async move {
                                match an_api::schedule_report(req).await {
                                    Ok(_) => {
                                        set_status.set(Some("PDF export queued — refresh to download".into()));
                                        reports.refetch();
                                    }
                                    Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
                                }
                            });
                        }
                    >
                        "Export PDF"
                    </button>
                </div>
                {move || status.get().map(|s| view! {
                    <div style="margin-top:10px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
                <p style="margin-top:14px;font-size:10px;color:#A0A0B0;">
                    "Download token is single-use — click Download immediately, the URL stops working after first use."
                </p>
            </div>
        </div>
    }
}
