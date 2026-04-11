//! Administrator pages — backup history, restore sandbox, retention cleanup,
//! user management, and the immutable audit log viewer.

use leptos::*;

pub mod backup;
pub mod users;
pub mod audit;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdminTab {
    Backup,
    Users,
    Audit,
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (tab, set_tab) = create_signal(AdminTab::Backup);

    view! {
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">"Administrator"</h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">"Backup, restore, retention, audit, and user management."</p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            <div style="display:flex;gap:8px;border-bottom:1px solid rgba(245,197,24,0.20);margin-bottom:24px;">
                {tab_btn("Backup & Restore", AdminTab::Backup, tab, set_tab)}
                {tab_btn("Users", AdminTab::Users, tab, set_tab)}
                {tab_btn("Audit Log", AdminTab::Audit, tab, set_tab)}
            </div>

            {move || match tab.get() {
                AdminTab::Backup => view! { <backup::BackupTab /> }.into_view(),
                AdminTab::Users => view! { <users::UsersTab /> }.into_view(),
                AdminTab::Audit => view! { <audit::AuditTab /> }.into_view(),
            }}
        </div>
    }
}

fn tab_btn(
    label: &'static str,
    this_tab: AdminTab,
    current: ReadSignal<AdminTab>,
    setter: WriteSignal<AdminTab>,
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
