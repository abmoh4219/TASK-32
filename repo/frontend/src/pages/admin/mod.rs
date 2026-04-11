//! Administrator pages — backup history, restore sandbox, retention cleanup,
//! user management, and the immutable audit log viewer.

use leptos::*;

use crate::components::layout::{NavTarget, PageShell};

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
        <PageShell
            active=NavTarget::Admin
            title="Administration"
            subtitle="Backup, restore, retention, audit, and user management"
        >
            <div class="sv-page-header">
                <h1 class="sv-page-title">"Administrator"</h1>
                <p class="sv-page-subtitle">"Backup & restore, retention policies, user management, and the immutable audit log."</p>
            </div>

            <div class="sv-tabs">
                {tab_btn("Backup & Restore", AdminTab::Backup, tab, set_tab)}
                {tab_btn("Users", AdminTab::Users, tab, set_tab)}
                {tab_btn("Audit Log", AdminTab::Audit, tab, set_tab)}
            </div>

            {move || match tab.get() {
                AdminTab::Backup => view! { <backup::BackupTab /> }.into_view(),
                AdminTab::Users => view! { <users::UsersTab /> }.into_view(),
                AdminTab::Audit => view! { <audit::AuditTab /> }.into_view(),
            }}
        </PageShell>
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
            class=move || if current.get() == this_tab { "sv-tab active" } else { "sv-tab" }
            on:click=move |_| setter.set(this_tab)
        >
            {label}
        </button>
    }
}
