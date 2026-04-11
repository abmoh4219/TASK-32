//! Login page — golden-gradient dark card centered on the screen with username +
//! password inputs, gold focus rings, primary login button with loading spinner,
//! and inline error display (including the lockout countdown message).

use leptos::*;
use leptos_router::use_navigate;
use wasm_bindgen_futures::spawn_local;

use crate::api::auth as auth_api;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (loading, set_loading) = create_signal(false);
    let (error, set_error) = create_signal::<Option<String>>(None);

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_loading.set(true);
        let u = username.get();
        let p = password.get();
        spawn_local(async move {
            match auth_api::login(&u, &p).await {
                Ok(resp) => {
                    set_loading.set(false);
                    let role = resp.role.clone();
                    let target = match role.as_str() {
                        "administrator" => "/admin",
                        "content_curator" => "/knowledge",
                        "reviewer" => "/outcomes",
                        "finance_manager" => "/analytics",
                        "store_manager" => "/store",
                        _ => "/dashboard",
                    };
                    let nav = use_navigate();
                    nav(target, Default::default());
                }
                Err(err) => {
                    set_loading.set(false);
                    set_error.set(Some(err.message));
                }
            }
        });
    };

    view! {
        <div style="min-height:100vh;display:flex;align-items:center;justify-content:center;background:radial-gradient(circle at top, #1A1A28 0%, #0A0A0F 70%);padding:20px;">
            <div class="sv-card" style="width:100%;max-width:420px;padding:40px;border-left:none;border:1px solid rgba(245,197,24,0.20);box-shadow:0 0 60px rgba(245,197,24,0.20);">
                <div style="text-align:center;margin-bottom:32px;">
                    <h1 class="sv-text-gradient" style="font-size:32px;font-weight:800;margin:0;letter-spacing:-0.5px;">
                        "ScholarVault"
                    </h1>
                    <p style="color:#A0A0B0;margin:8px 0 0;font-size:13px;">
                        "Research & Commerce Operations Portal"
                    </p>
                </div>

                <form on:submit=on_submit>
                    <div style="margin-bottom:18px;">
                        <label class="sv-label">"Username"</label>
                        <input
                            class="sv-input"
                            type="text"
                            autocomplete="username"
                            prop:value=move || username.get()
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            placeholder="admin"
                            required="required"
                        />
                    </div>

                    <div style="margin-bottom:24px;">
                        <label class="sv-label">"Password"</label>
                        <input
                            class="sv-input"
                            type="password"
                            autocomplete="current-password"
                            prop:value=move || password.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            placeholder="••••••••"
                            required="required"
                        />
                    </div>

                    {move || error.get().map(|msg| view! {
                        <div style="background:rgba(239,68,68,0.10);border:1px solid rgba(239,68,68,0.40);border-radius:8px;padding:12px 14px;margin-bottom:18px;color:#fca5a5;font-size:13px;">
                            {msg}
                        </div>
                    })}

                    <button
                        class="sv-btn-primary"
                        type="submit"
                        style="width:100%;padding:14px;font-size:15px;"
                        prop:disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Signing in…" } else { "Sign In" }}
                    </button>
                </form>

                <div style="margin-top:28px;padding-top:20px;border-top:1px solid rgba(245,197,24,0.10);font-size:11px;color:#A0A0B0;">
                    <div style="margin-bottom:6px;font-weight:600;color:#F5C518;text-transform:uppercase;letter-spacing:0.05em;">"Sample Logins"</div>
                    <div>"admin / ScholarAdmin2024!"</div>
                    <div>"curator | reviewer | finance | store / Scholar2024!"</div>
                </div>
            </div>
        </div>
    }
}
