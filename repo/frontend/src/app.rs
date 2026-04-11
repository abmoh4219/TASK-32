//! Root Leptos component. Wires up the router with all top-level routes.
//! Phase 0 ships a minimal placeholder home page; subsequent phases register
//! their pages by adding `<Route>` entries below.

use leptos::*;
use leptos_router::{Route, Router, Routes};

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="min-h-screen" style="background:#0A0A0F;color:#F0F0F5;">
                <Routes>
                    <Route path="/" view=Home/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div style="display:flex;align-items:center;justify-content:center;min-height:100vh;">
            <div style="text-align:center;">
                <h1 style="font-size:42px;font-weight:800;margin:0;background:linear-gradient(135deg,#F5C518,#E8A900,#CC8800);-webkit-background-clip:text;background-clip:text;color:transparent;">
                    "ScholarVault"
                </h1>
                <p style="color:#A0A0B0;margin-top:8px;">"Research & Commerce Operations Portal"</p>
                <p style="color:#A0A0B0;margin-top:24px;font-size:12px;">"Phase 0 bootstrap"</p>
            </div>
        </div>
    }
}
