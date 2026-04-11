//! Audit log tab — read-only viewer for the immutable `audit_logs` table.
//! There is no edit / delete UI by design: the table is append-only and
//! marked IMMUTABLE in the page header.

use leptos::*;
use serde::{Deserialize, Serialize};

use crate::api::client::{get_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub actor_id: String,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub before_hash: Option<String>,
    pub after_hash: Option<String>,
    pub ip_address: Option<String>,
    pub created_at: String,
}

async fn fetch_audit() -> Result<Vec<AuditLog>, ApiError> {
    get_json("/api/admin/audit").await
}

#[component]
pub fn AuditTab() -> impl IntoView {
    let logs = create_resource(|| (), |_| async move { fetch_audit().await });

    view! {
        <div class="sv-card">
            <div style="display:flex;align-items:center;gap:12px;margin-bottom:14px;">
                <h2 style="margin:0;font-size:16px;color:#F5C518;">"Audit Log"</h2>
                <span class="sv-badge sv-badge-warning">"IMMUTABLE — append-only"</span>
            </div>
            <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:200px;"></div> }>
                {move || logs.get().map(|res| match res {
                    Ok(rows) if rows.is_empty() => view! {
                        <div style="text-align:center;color:#A0A0B0;padding:24px;">"No audit entries yet."</div>
                    }.into_view(),
                    Ok(rows) => view! {
                        <table class="sv-table">
                            <thead>
                                <tr>
                                    <th>"When"</th>
                                    <th>"Actor"</th>
                                    <th>"Action"</th>
                                    <th>"Entity"</th>
                                    <th>"Before hash"</th>
                                    <th>"After hash"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {rows.into_iter().map(|l| view! {
                                    <tr>
                                        <td style="font-size:11px;color:#A0A0B0;">{l.created_at}</td>
                                        <td style="font-family:monospace;font-size:11px;">{l.actor_id}</td>
                                        <td><span class="sv-badge sv-badge-info">{l.action}</span></td>
                                        <td style="font-size:11px;color:#A0A0B0;">
                                            {format!("{}/{}",
                                                l.entity_type.unwrap_or_else(|| "—".into()),
                                                l.entity_id.unwrap_or_else(|| "—".into()))}
                                        </td>
                                        <td style="font-family:monospace;font-size:10px;color:#A0A0B0;">
                                            {l.before_hash.map(|h| h.chars().take(12).collect::<String>()).unwrap_or_default()}
                                        </td>
                                        <td style="font-family:monospace;font-size:10px;color:#A0A0B0;">
                                            {l.after_hash.map(|h| h.chars().take(12).collect::<String>()).unwrap_or_default()}
                                        </td>
                                    </tr>
                                }).collect_view()}
                            </tbody>
                        </table>
                    }.into_view(),
                    Err(e) => view! { <div class="sv-error">{e.message}</div> }.into_view(),
                })}
            </Suspense>
        </div>
    }
}
