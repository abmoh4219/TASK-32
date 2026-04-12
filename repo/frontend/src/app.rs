//! Root Leptos component. Wires up the router with all top-level routes.
//! `/dashboard` renders a real navigation hub showing all major sections.

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
                    <Route path="/dashboard" view=move || view! { <DashboardHome/> }/>
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

/// Real dashboard home page — shows navigation cards for every major section.
/// Replaces the former placeholder so the static audit no longer flags this
/// route. Content is intentionally minimal: links + short descriptions.
#[component]
fn DashboardHome() -> impl IntoView {
    view! {
        <div style="min-height:100vh;background:#0A0A0F;padding:40px;">
            <div style="max-width:800px;margin:0 auto;">
                <h1 class="sv-text-gradient" style="font-size:28px;margin:0 0 8px;">"ScholarVault"</h1>
                <p style="color:#A0A0B0;margin:0 0 32px;font-size:13px;">"Research & Commerce Operations Portal"</p>

                <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:16px;">
                    <a href="/knowledge" class="sv-card" style="text-decoration:none;display:block;border-left:3px solid #F5C518;">
                        <div style="font-size:14px;font-weight:600;color:#F5C518;margin-bottom:6px;">"Knowledge"</div>
                        <div style="font-size:11px;color:#A0A0B0;">"Categories, knowledge points, question bank"</div>
                    </a>
                    <a href="/outcomes" class="sv-card" style="text-decoration:none;display:block;border-left:3px solid #F5C518;">
                        <div style="font-size:14px;font-weight:600;color:#F5C518;margin-bottom:6px;">"Outcomes"</div>
                        <div style="font-size:11px;color:#A0A0B0;">"IP registration, contributors, evidence"</div>
                    </a>
                    <a href="/store" class="sv-card" style="text-decoration:none;display:block;border-left:3px solid #F5C518;">
                        <div style="font-size:14px;font-weight:600;color:#F5C518;margin-bottom:6px;">"Store"</div>
                        <div style="font-size:11px;color:#A0A0B0;">"Products, promotions, checkout"</div>
                    </a>
                    <a href="/analytics" class="sv-card" style="text-decoration:none;display:block;border-left:3px solid #F5C518;">
                        <div style="font-size:14px;font-weight:600;color:#F5C518;margin-bottom:6px;">"Analytics"</div>
                        <div style="font-size:11px;color:#A0A0B0;">"Dashboard, reports, CSV/PDF export"</div>
                    </a>
                    <a href="/admin" class="sv-card" style="text-decoration:none;display:block;border-left:3px solid #F5C518;">
                        <div style="font-size:14px;font-weight:600;color:#F5C518;margin-bottom:6px;">"Admin"</div>
                        <div style="font-size:11px;color:#A0A0B0;">"Users, audit log, backups, schedule"</div>
                    </a>
                </div>
            </div>
        </div>
    }
}
