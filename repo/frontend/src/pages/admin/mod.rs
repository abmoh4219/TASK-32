//! Administrator pages — backup history, restore sandbox, retention cleanup.
//! Phase 8 will add a Users tab and an Audit Log tab to this page.

use leptos::*;

pub mod backup;

#[derive(Clone, Copy, PartialEq, Eq)]
enum AdminTab {
    Backup,
}

#[component]
pub fn AdminPage() -> impl IntoView {
    let (tab, _set_tab) = create_signal(AdminTab::Backup);

    view! {
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">"Administrator"</h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">"Backup, restore, retention, audit, and user management."</p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            {move || match tab.get() {
                AdminTab::Backup => view! { <backup::BackupTab /> }.into_view(),
            }}
        </div>
    }
}
