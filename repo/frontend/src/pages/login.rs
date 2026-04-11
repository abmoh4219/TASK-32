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
        <div class="sv-login-bg">
            <div class="sv-login-card">
                <div class="sv-login-logo">
                    <div class="sv-login-mark">"ScholarVault"</div>
                    <div class="sv-login-tag">"Research · Commerce · IP"</div>
                </div>

                <form on:submit=on_submit>
                    <div class="sv-field">
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

                    <div class="sv-field">
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
                        <div class="sv-error-banner">{msg}</div>
                    })}

                    <button
                        class="sv-btn-primary sv-btn-full"
                        type="submit"
                        style="padding:14px;font-size:14px;margin-top:6px;letter-spacing:0.04em;text-transform:uppercase;"
                        prop:disabled=move || loading.get()
                    >
                        {move || if loading.get() { "Signing in…" } else { "Sign in" }}
                    </button>
                </form>

                <div style="margin-top:28px;padding-top:18px;border-top:1px solid rgba(245,197,24,0.10);text-align:center;">
                    <div style="font-size:11px;color:var(--text-muted);letter-spacing:0.04em;">
                        "Secured with Argon2id · AES-256-GCM · CSRF"
                    </div>
                </div>
            </div>
        </div>
    }
}
