//! Users tab — admin can list users, create new ones (with role select), and
//! change roles. Calls `/api/admin/users*` which is gated by `RequireAdmin`.

use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen_futures::spawn_local;

use crate::api::client::{get_json, post_json, ApiError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSummary {
    pub id: String,
    pub username: String,
    pub role: String,
    pub is_active: i64,
    pub full_name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
struct CreateUserBody {
    username: String,
    password: String,
    role: String,
    full_name: Option<String>,
    email: Option<String>,
}

async fn list_users() -> Result<Vec<UserSummary>, ApiError> {
    get_json("/api/admin/users").await
}

async fn create_user(body: CreateUserBody) -> Result<UserSummary, ApiError> {
    post_json("/api/admin/users", &body).await
}

#[component]
pub fn UsersTab() -> impl IntoView {
    let users = create_resource(|| (), |_| async move { list_users().await });
    let (status, set_status) = create_signal::<Option<String>>(None);
    let (new_username, set_new_username) = create_signal(String::new());
    let (new_password, set_new_password) = create_signal(String::new());
    let (new_role, set_new_role) = create_signal("content_curator".to_string());

    let create = move |_| {
        let body = CreateUserBody {
            username: new_username.get(),
            password: new_password.get(),
            role: new_role.get(),
            full_name: None,
            email: None,
        };
        if body.username.is_empty() || body.password.len() < 8 {
            set_status.set(Some("Username + 8-char password required".into()));
            return;
        }
        spawn_local(async move {
            match create_user(body).await {
                Ok(u) => {
                    set_status.set(Some(format!("Created {}", u.username)));
                    set_new_username.set(String::new());
                    set_new_password.set(String::new());
                    users.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1.6fr 1fr;gap:24px;">
            <div class="sv-card">
                <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"All Users"</h2>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                    {move || users.get().map(|res| match res {
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead>
                                    <tr>
                                        <th>"Username"</th>
                                        <th>"Role"</th>
                                        <th>"Status"</th>
                                        <th>"Created"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|u| {
                                        let active_badge = if u.is_active == 1 {
                                            ("sv-badge sv-badge-success", "active")
                                        } else {
                                            ("sv-badge sv-badge-danger", "inactive")
                                        };
                                        view! {
                                            <tr>
                                                <td>
                                                    <div style="font-weight:600;">{u.username}</div>
                                                    <div style="font-size:11px;color:#A0A0B0;font-family:monospace;">{u.id}</div>
                                                </td>
                                                <td><span class="sv-badge sv-badge-info">{u.role}</span></td>
                                                <td><span class=active_badge.0>{active_badge.1}</span></td>
                                                <td style="color:#A0A0B0;font-size:11px;">{u.created_at}</td>
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
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Create User"</h3>
                <label class="sv-label">"Username"</label>
                <input class="sv-input" prop:value=move || new_username.get()
                    on:input=move |ev| set_new_username.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Password (≥ 8 chars)"</label>
                <input class="sv-input" type="password" prop:value=move || new_password.get()
                    on:input=move |ev| set_new_password.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Role"</label>
                <select class="sv-input" on:change=move |ev| set_new_role.set(event_target_value(&ev))>
                    <option value="content_curator">"Content Curator"</option>
                    <option value="reviewer">"Reviewer"</option>
                    <option value="finance_manager">"Finance Manager"</option>
                    <option value="store_manager">"Store Manager"</option>
                    <option value="administrator">"Administrator"</option>
                </select>
                <button class="sv-btn-primary" style="margin-top:14px;width:100%;" on:click=create>
                    "Create User"
                </button>
                {move || status.get().map(|s| view! {
                    <div style="margin-top:10px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
            </div>
        </div>
    }
}
