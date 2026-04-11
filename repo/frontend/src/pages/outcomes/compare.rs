//! Side-by-side compare-two-outcomes view. Calls
//! `/api/outcomes/:id/compare/:other_id` and highlights fields that differ.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::outcomes::{self as out_api, CompareResult};

#[component]
pub fn CompareOutcomes() -> impl IntoView {
    let (id_a, set_id_a) = create_signal(String::new());
    let (id_b, set_id_b) = create_signal(String::new());
    let (result, set_result) = create_signal::<Option<CompareResult>>(None);
    let (status, set_status) = create_signal::<Option<String>>(None);

    let load = move |_| {
        let a = id_a.get();
        let b = id_b.get();
        if a.is_empty() || b.is_empty() {
            set_status.set(Some("Provide both ids".into()));
            return;
        }
        spawn_local(async move {
            match out_api::compare_outcomes(&a, &b).await {
                Ok(r) => {
                    set_result.set(Some(r));
                    set_status.set(Some("Loaded".into()));
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div>
            <div class="sv-card" style="margin-bottom:16px;">
                <div style="display:grid;grid-template-columns:1fr 1fr auto;gap:10px;align-items:end;">
                    <div>
                        <label class="sv-label">"Outcome A id"</label>
                        <input class="sv-input" prop:value=move || id_a.get()
                            on:input=move |ev| set_id_a.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"Outcome B id"</label>
                        <input class="sv-input" prop:value=move || id_b.get()
                            on:input=move |ev| set_id_b.set(event_target_value(&ev))/>
                    </div>
                    <button class="sv-btn-primary" on:click=load>"Compare"</button>
                </div>
                {move || status.get().map(|s| view! {
                    <div style="margin-top:10px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
            </div>

            {move || result.get().map(|r| {
                let title_diff = r.a.title != r.b.title;
                let abstract_diff = r.a.abstract_snippet != r.b.abstract_snippet;
                let cert_diff = r.a.certificate_number != r.b.certificate_number;
                let r_a = r.a.clone();
                let r_b = r.b.clone();
                view! {
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:24px;">
                        {render_card("A", r_a, title_diff, abstract_diff, cert_diff)}
                        {render_card("B", r_b, title_diff, abstract_diff, cert_diff)}
                    </div>
                    <div class="sv-card" style="margin-top:18px;background:rgba(245,197,24,0.05);">
                        <div style="font-size:13px;color:#F5C518;font-weight:600;">
                            {format!("Similarity → title {:.2}, abstract {:.2}", r.title_similarity, r.abstract_similarity)}
                        </div>
                    </div>
                }
            })}
        </div>
    }
}

fn render_card(
    label: &'static str,
    o: out_api::Outcome,
    title_diff: bool,
    abstract_diff: bool,
    cert_diff: bool,
) -> impl IntoView {
    let highlight = |diff: bool| if diff { "background:rgba(245,158,11,0.10);padding:8px;border-radius:4px;" } else { "padding:8px;" };
    view! {
        <div class="sv-card">
            <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">{format!("Outcome {}", label)}</h3>
            <div style="font-size:11px;color:#A0A0B0;font-family:monospace;">{o.id.clone()}</div>
            <div style="margin-top:12px;">
                <label class="sv-label">"Type"</label>
                <div style="font-size:13px;">{o.r#type.clone()}</div>
            </div>
            <div style="margin-top:8px;">
                <label class="sv-label">"Title"</label>
                <div style=highlight(title_diff)>{o.title.clone()}</div>
            </div>
            <div style="margin-top:8px;">
                <label class="sv-label">"Abstract"</label>
                <div style=highlight(abstract_diff)>{o.abstract_snippet.clone()}</div>
            </div>
            <div style="margin-top:8px;">
                <label class="sv-label">"Certificate"</label>
                <div style=highlight(cert_diff)>{o.certificate_number.clone().unwrap_or_else(|| "—".into())}</div>
            </div>
        </div>
    }
}
