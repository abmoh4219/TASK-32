//! Root Leptos component. Wires up the router with all top-level routes.
//! Phase 2 hooks `/login` and a temporary placeholder for the role dashboards
//! that the later phases will replace with real screens.

use leptos::*;
use leptos_router::{Route, Router, Routes};

use crate::pages::login::LoginPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main style="min-height:100vh;background:#0A0A0F;color:#F0F0F5;">
                <Routes>
                    <Route path="/" view=move || view! { <LoginPage/> }/>
                    <Route path="/login" view=move || view! { <LoginPage/> }/>
                    <Route path="/dashboard" view=move || view! { <PlaceholderDashboard label="User".to_string()/> }/>
                    <Route path="/admin" view=move || view! { <PlaceholderDashboard label="Administrator".to_string()/> }/>
                    <Route path="/knowledge" view=move || view! { <PlaceholderDashboard label="Content Curator".to_string()/> }/>
                    <Route path="/outcomes" view=move || view! { <PlaceholderDashboard label="Reviewer".to_string()/> }/>
                    <Route path="/analytics" view=move || view! { <PlaceholderDashboard label="Finance Manager".to_string()/> }/>
                    <Route path="/store" view=move || view! { <PlaceholderDashboard label="Store Manager".to_string()/> }/>
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
