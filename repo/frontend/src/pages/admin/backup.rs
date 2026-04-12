//! Backup tab — list history, run a backup, restore via sandbox validation,
//! lifecycle cleanup, retention policy display.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::backup as backup_api;

#[component]
pub fn BackupTab() -> impl IntoView {
    let history = create_resource(|| (), |_| async move { backup_api::list_history().await });
    let policy = create_resource(|| (), |_| async move { backup_api::get_policy().await });
    let schedule = create_resource(|| (), |_| async move { backup_api::get_schedule().await });
    let (status, set_status) = create_signal::<Option<String>>(None);
    let (cron_draft, set_cron_draft) = create_signal(String::new());
    let (daily_draft, set_daily_draft) = create_signal(30i64);
    let (monthly_draft, set_monthly_draft) = create_signal(12i64);
    let (pf_draft, set_pf_draft) = create_signal(true);
    let (pi_draft, set_pi_draft) = create_signal(true);

    let save_schedule = move |_| {
        let expr = cron_draft.get();
        if expr.trim().is_empty() {
            set_status.set(Some("Cron expression is required".into()));
            return;
        }
        let schedule_res = schedule;
        spawn_local(async move {
            match backup_api::update_schedule(backup_api::UpdateScheduleRequest {
                cron_expr: expr,
            })
            .await
            {
                Ok(_) => {
                    set_status.set(Some("Schedule updated".into()));
                    schedule_res.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let save_policy = move |_| {
        let policy_res = policy;
        spawn_local(async move {
            match backup_api::update_policy(backup_api::UpdatePolicyRequest {
                daily_retention: daily_draft.get(),
                monthly_retention: monthly_draft.get(),
                preserve_financial: pf_draft.get(),
                preserve_ip: pi_draft.get(),
            })
            .await
            {
                Ok(_) => {
                    set_status.set(Some("Retention policy updated".into()));
                    policy_res.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };
    let (validation, set_validation) =
        create_signal::<Option<backup_api::SandboxValidationReport>>(None);
    let (validating_id, set_validating_id) = create_signal::<Option<String>>(None);

    let run_backup = move |_| {
        spawn_local(async move {
            match backup_api::run_backup().await {
                Ok(b) => {
                    set_status.set(Some(format!("Created {} backup {}", b.backup_type, &b.id[..8])));
                    history.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let cleanup = move |_| {
        spawn_local(async move {
            match backup_api::lifecycle_cleanup().await {
                Ok(r) => set_status.set(Some(format!(
                    "Purged {} daily / {} monthly • preserved {} fin / {} ip",
                    r.purged_daily, r.purged_monthly, r.preserved_financial, r.preserved_ip
                ))),
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let validate = move |id: String| {
        set_validating_id.set(Some(id.clone()));
        spawn_local(async move {
            match backup_api::restore_sandbox(&id).await {
                Ok(r) => set_validation.set(Some(r)),
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let activate = move |id: String| {
        spawn_local(async move {
            match backup_api::activate(&id).await {
                Ok(_) => set_status.set(Some(format!("Activated restore for {}", &id[..8]))),
                Err(e) => set_status.set(Some(format!("Activate failed: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1.6fr 1fr;gap:24px;">
            <div class="sv-card">
                <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:14px;">
                    <h2 style="margin:0;font-size:16px;color:#F5C518;">"Backup History"</h2>
                    <div style="display:flex;gap:8px;">
                        <button class="sv-btn-primary" on:click=run_backup>"Run Backup Now"</button>
                        <button class="sv-btn-secondary" on:click=cleanup>"Lifecycle Cleanup"</button>
                    </div>
                </div>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                    {move || history.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div style="text-align:center;color:#A0A0B0;padding:24px;">"No backups yet — click Run Backup Now."</div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead>
                                    <tr><th>"Type"</th><th>"Created"</th><th>"Size"</th><th>"Status"</th><th>"SHA-256"</th><th>"Actions"</th></tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|r| {
                                        let badge = match r.status.as_str() {
                                            "complete" => "sv-badge sv-badge-success",
                                            "purged" => "sv-badge sv-badge-warning",
                                            _ => "sv-badge sv-badge-info",
                                        };
                                        let id_for_validate = r.id.clone();
                                        let id_for_activate = r.id.clone();
                                        view! {
                                            <tr>
                                                <td><span class="sv-badge sv-badge-info">{r.backup_type.clone()}</span></td>
                                                <td style="font-size:11px;color:#A0A0B0;">{r.created_at.clone()}</td>
                                                <td>{format!("{} KB", r.size_bytes / 1024)}</td>
                                                <td><span class=badge>{r.status.clone()}</span></td>
                                                <td style="font-family:monospace;font-size:10px;">{format!("{}…", &r.sha256_hash[..12])}</td>
                                                <td>
                                                    <button class="sv-btn-ghost" style="font-size:11px;padding:4px 8px;"
                                                        on:click=move |_| validate(id_for_validate.clone())>"Validate"</button>
                                                    <button class="sv-btn-ghost" style="font-size:11px;padding:4px 8px;"
                                                        on:click=move |_| activate(id_for_activate.clone())>"Activate"</button>
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
                {move || status.get().map(|s| view! {
                    <div style="margin-top:14px;font-size:12px;color:#A0A0B0;">{s}</div>
                })}
            </div>

            <div style="display:flex;flex-direction:column;gap:16px;">
                <div class="sv-card">
                    <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Retention Policy"</h3>
                    <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:80px;"></div> }>
                        {move || policy.get().map(|res| match res {
                            Ok(p) => view! {
                                <div style="font-size:12px;line-height:1.8;">
                                    <div>{format!("Daily backups kept: {}", p.daily_retention)}</div>
                                    <div>{format!("Monthly backups kept: {}", p.monthly_retention)}</div>
                                    <div>{format!("Preserve financial: {}", if p.preserve_financial == 1 { "yes" } else { "no" })}</div>
                                    <div>{format!("Preserve IP: {}", if p.preserve_ip == 1 { "yes" } else { "no" })}</div>
                                </div>
                            }.into_view(),
                            Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                        })}
                    </Suspense>
                    <div style="margin-top:14px;border-top:1px solid rgba(245,197,24,0.10);padding-top:12px;">
                        <label class="sv-label">"Daily"</label>
                        <input class="sv-input" type="number" min="1" max="3650"
                            prop:value=move || daily_draft.get().to_string()
                            on:input=move |ev| set_daily_draft.set(event_target_value(&ev).parse().unwrap_or(30))/>
                        <label class="sv-label" style="margin-top:8px;">"Monthly"</label>
                        <input class="sv-input" type="number" min="1" max="120"
                            prop:value=move || monthly_draft.get().to_string()
                            on:input=move |ev| set_monthly_draft.set(event_target_value(&ev).parse().unwrap_or(12))/>
                        <label style="display:flex;gap:6px;align-items:center;margin-top:8px;font-size:11px;">
                            <input type="checkbox"
                                prop:checked=move || pf_draft.get()
                                on:change=move |ev| set_pf_draft.set(event_target_checked(&ev))/>
                            "Preserve financial"
                        </label>
                        <label style="display:flex;gap:6px;align-items:center;margin-top:4px;font-size:11px;">
                            <input type="checkbox"
                                prop:checked=move || pi_draft.get()
                                on:change=move |ev| set_pi_draft.set(event_target_checked(&ev))/>
                            "Preserve IP"
                        </label>
                        <button class="sv-btn-primary" style="margin-top:10px;width:100%;" on:click=save_policy>
                            "Save Policy"
                        </button>
                    </div>
                </div>

                <div class="sv-card">
                    <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Backup Schedule (cron)"</h3>
                    <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:60px;"></div> }>
                        {move || schedule.get().map(|res| match res {
                            Ok(s) => view! {
                                <div style="font-size:12px;color:#A0A0B0;margin-bottom:8px;">
                                    {format!("Current: {}", s.cron_expr)}
                                </div>
                            }.into_view(),
                            Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                        })}
                    </Suspense>
                    <label class="sv-label">"New cron (5-7 fields)"</label>
                    <input class="sv-input" placeholder="0 0 2 * * *"
                        prop:value=move || cron_draft.get()
                        on:input=move |ev| set_cron_draft.set(event_target_value(&ev))/>
                    <button class="sv-btn-primary" style="margin-top:10px;width:100%;" on:click=save_schedule>
                        "Save Schedule"
                    </button>
                </div>

                {move || validation.get().map(|r| {
                    let id = validating_id.get().unwrap_or_default();
                    let badge = if r.all_passed { "sv-badge sv-badge-success" } else { "sv-badge sv-badge-danger" };
                    view! {
                        <div class="sv-card">
                            <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">{format!("Sandbox: {}", id)}</h3>
                            <div style="display:flex;flex-direction:column;gap:6px;font-size:12px;">
                                <div>{format!("Hash check: {}", if r.hash_ok { "✓" } else { "✗" })}</div>
                                <div>{format!("PRAGMA integrity_check: {}", if r.integrity_ok { "✓" } else { "✗" })}</div>
                                <div>{format!("Read test (SELECT users): {}", if r.read_test_ok { "✓" } else { "✗" })}</div>
                                <div style="margin-top:8px;"><span class=badge>{if r.all_passed { "ALL PASSED" } else { "BLOCKED" }}</span></div>
                                <div style="margin-top:6px;color:#A0A0B0;font-size:11px;">{r.message}</div>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
