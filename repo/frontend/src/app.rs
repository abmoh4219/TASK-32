//! Root Leptos component. Wires up the router with all top-level routes.
//! Phase 2 hooks `/login` and a temporary placeholder for the role dashboards
//! that the later phases will replace with real screens.

use leptos::*;
use leptos_router::{Route, Router, Routes};

use crate::pages::admin::AdminPage;
use crate::pages::analytics::AnalyticsPage;
use crate::pages::knowledge::KnowledgePage;
use crate::pages::login::LoginPage;
use crate::pages::outcomes::OutcomesPage;
use crate::pages::store::StorePage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main style="min-height:100vh;background:#0A0A0F;color:#F0F0F5;">
                <Routes>
                    <Route path="/" view=move || view! { <LoginPage/> }/>
                    <Route path="/login" view=move || view! { <LoginPage/> }/>
                    <Route path="/dashboard" view=move || view! { <PlaceholderDashboard label="User".to_string()/> }/>
                    <Route path="/admin" view=AdminPage/>
                    <Route path="/knowledge" view=KnowledgePage/>
                    <Route path="/outcomes" view=OutcomesPage/>
                    <Route path="/analytics" view=AnalyticsPage/>
                    <Route path="/store" view=StorePage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn PlaceholderDashboard(#[prop(optional)] label: String) -> impl IntoView {
    let role_text = if label.is_empty() { "Authenticated".to_string() } else { label };
    view! {
        <div style="min-height:100vh;display:flex;align-items:center;justify-content:center;">
            <div class="sv-card" style="text-align:center;padding:40px;max-width:520px;">
                <h1 class="sv-text-gradient" style="font-size:28px;margin:0 0 12px;">
                    "ScholarVault"
                </h1>
                <p style="color:#A0A0B0;margin:0 0 18px;">
                    {format!("Logged in as {}", role_text)}
                </p>
                <p style="color:#A0A0B0;font-size:12px;margin:0;">
                    "Role-specific pages are wired up by Phases 3–8."
                </p>
            </div>
        </div>
    }
}
