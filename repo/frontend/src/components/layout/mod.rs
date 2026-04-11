//! Layout chrome — sidebar + topbar + `PageShell` used by every authenticated
//! page. Pure presentation; no backend or business logic touched.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::auth as auth_api;
use crate::api::client::post_json;

/// Navigation entries that can be rendered in the sidebar. `NavTarget` is a
/// lightweight tag used by every page to mark which item is currently active.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavTarget {
    Dashboard,
    Knowledge,
    Outcomes,
    Store,
    Analytics,
    Admin,
}

#[derive(Clone)]
struct NavEntry {
    label: &'static str,
    icon: &'static str,
    href: &'static str,
    target: NavTarget,
    /// Roles allowed to see (and navigate to) this entry. The sidebar filters
    /// the list live against the value returned by `/api/auth/me` so every
    /// role only sees the pages it can actually access — no disabled or greyed
    /// out links, no forbidden-error surprises.
    allowed: &'static [&'static str],
}

const ROLE_ALL: &[&str] = &[
    "administrator",
    "content_curator",
    "reviewer",
    "finance_manager",
    "store_manager",
];

const NAV: &[NavEntry] = &[
    NavEntry {
        label: "Dashboard",
        icon: "◆",
        href: "/dashboard",
        target: NavTarget::Dashboard,
        allowed: ROLE_ALL,
    },
    NavEntry {
        label: "Knowledge",
        icon: "◈",
        href: "/knowledge",
        target: NavTarget::Knowledge,
        allowed: &["administrator", "content_curator"],
    },
    NavEntry {
        label: "Outcomes",
        icon: "◉",
        href: "/outcomes",
        target: NavTarget::Outcomes,
        allowed: &["administrator", "reviewer"],
    },
    NavEntry {
        label: "Store",
        icon: "◎",
        href: "/store",
        target: NavTarget::Store,
        allowed: &["administrator", "store_manager"],
    },
    NavEntry {
        label: "Analytics",
        icon: "◐",
        href: "/analytics",
        target: NavTarget::Analytics,
        allowed: &["administrator", "finance_manager"],
    },
    NavEntry {
        label: "Administrator",
        icon: "◇",
        href: "/admin",
        target: NavTarget::Admin,
        allowed: &["administrator"],
    },
];

fn role_display_name(role: &str) -> &'static str {
    match role {
        "administrator" => "Administrator",
        "content_curator" => "Content Curator",
        "reviewer" => "Reviewer",
        "finance_manager" => "Finance Manager",
        "store_manager" => "Store Manager",
        _ => "User",
    }
}

/// Full-page shell: golden-gradient sidebar on the left, sticky topbar with
/// the page title + breadcrumbs on top, and the caller's children rendered
/// inside the `.sv-content` area.
#[component]
pub fn PageShell(
    active: NavTarget,
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    children: Children,
) -> impl IntoView {
    let me = create_resource(|| (), |_| async move { auth_api::me().await });

    let do_logout = move |_| {
        spawn_local(async move {
            let _: Result<serde_json::Value, _> =
                post_json("/api/auth/logout", &serde_json::json!({})).await;
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/login");
                }
            }
        });
    };

    view! {
        <div class="sv-app">
            <aside class="sv-sidebar">
                <div class="sv-sidebar-logo">
                    <span class="sv-sidebar-logo-text">"ScholarVault"</span>
                    <div class="sv-sidebar-logo-sub">"Research · Commerce · IP"</div>
                </div>

                <div class="sv-sidebar-section">"Workspace"</div>
                {move || {
                    // Only render nav items once we know the current role so
                    // forbidden links never flash on screen. While /api/auth/me
                    // is in flight the nav area stays empty.
                    let role = me.get()
                        .and_then(|r| r.ok())
                        .map(|m| m.role)
                        .unwrap_or_default();
                    if role.is_empty() {
                        return ().into_view();
                    }
                    NAV.iter()
                        .filter(|n| n.allowed.contains(&role.as_str()))
                        .map(|n| {
                            let is_active = n.target == active;
                            let class = if is_active {
                                "sv-sidebar-item active"
                            } else {
                                "sv-sidebar-item"
                            };
                            view! {
                                <a class=class href=n.href>
                                    <span class="sv-sidebar-icon">{n.icon}</span>
                                    <span>{n.label}</span>
                                </a>
                            }
                        })
                        .collect_view()
                }}

                <div class="sv-sidebar-footer">
                    {move || {
                        let user = me.get();
                        let (username, role, initial) = match user {
                            Some(Ok(ref m)) => (
                                m.username.clone(),
                                m.role.clone(),
                                m.username.chars().next().map(|c| c.to_ascii_uppercase().to_string()).unwrap_or_else(|| "?".into()),
                            ),
                            _ => ("loading".into(), "".into(), "·".into()),
                        };
                        let role_class = format!("sv-badge sv-role-{}", role);
                        view! {
                            <div class="sv-role-pill">
                                <div class="sv-role-avatar">{initial}</div>
                                <div class="sv-role-meta">
                                    <div class="sv-role-username">{username}</div>
                                    <div class="sv-role-label">
                                        {if role.is_empty() {
                                            "—".to_string()
                                        } else {
                                            role_display_name(&role).to_string()
                                        }}
                                    </div>
                                </div>
                            </div>
                            <span class=role_class style="display:none;"></span>
                        }
                    }}
                    <button class="sv-sidebar-logout" on:click=do_logout>"Sign out"</button>
                </div>
            </aside>

            <main class="sv-main">
                <header class="sv-topbar">
                    <div>
                        <div class="sv-topbar-title">{title.clone()}</div>
                        <div class="sv-topbar-crumbs">{subtitle.clone()}</div>
                    </div>
                    <div class="sv-topbar-right">
                        {move || me.get().and_then(|r| r.ok()).map(|m| {
                            let role_class = format!("sv-badge sv-role-{}", m.role);
                            let label = role_display_name(&m.role).to_string();
                            view! {
                                <span class=role_class>{label}</span>
                            }
                        })}
                    </div>
                </header>
                <div class="sv-content">
                    {children()}
                </div>
            </main>
        </div>
    }
}
