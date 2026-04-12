//! Question bank tab — list questions, show link status, and let curators
//! link a question to a knowledge point by id.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::knowledge as kn_api;

#[component]
pub fn QuestionBankTab() -> impl IntoView {
    let questions = create_resource(|| (), |_| async move { kn_api::list_questions().await });
    let (link_qid, set_link_qid) = create_signal(String::new());
    let (link_kp, set_link_kp) = create_signal(String::new());
    let (status, set_status) = create_signal::<Option<String>>(None);

    let link_action = move |_| {
        let qid = link_qid.get();
        let kp = link_kp.get();
        if qid.trim().is_empty() || kp.trim().is_empty() {
            set_status.set(Some("Question id and knowledge point id are required".into()));
            return;
        }
        let questions = questions;
        spawn_local(async move {
            match kn_api::link_question_to_kp(&qid, kp.clone()).await {
                Ok(_) => {
                    set_status.set(Some(format!("Linked {qid} → {kp}")));
                    questions.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1fr 320px;gap:24px;">
            <div class="sv-card">
                <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"Questions"</h2>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                    {move || questions.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div style="color:#A0A0B0;padding:24px 0;text-align:center;">
                                "No questions in the bank yet — link them from a knowledge point."
                            </div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead>
                                    <tr>
                                        <th>"Question"</th>
                                        <th>"Type"</th>
                                        <th>"Knowledge Point"</th>
                                        <th>"Chapter"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|q| view! {
                                        <tr>
                                            <td>{q.question_text}</td>
                                            <td><span class="sv-badge sv-badge-info">{q.question_type}</span></td>
                                            <td style="font-family:monospace;font-size:11px;color:#A0A0B0;">
                                                {q.knowledge_point_id.unwrap_or_else(|| "—".to_string())}
                                            </td>
                                            <td>{q.chapter.unwrap_or_else(|| "—".to_string())}</td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{format!("Failed: {}", e.message)}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div class="sv-card">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Link question to knowledge point"</h3>
                <label class="sv-label">"Question id"</label>
                <input
                    class="sv-input"
                    placeholder="q-uuid"
                    prop:value=move || link_qid.get()
                    on:input=move |ev| set_link_qid.set(event_target_value(&ev))
                />
                <label class="sv-label" style="margin-top:10px;">"Knowledge point id"</label>
                <input
                    class="sv-input"
                    placeholder="kp-001"
                    prop:value=move || link_kp.get()
                    on:input=move |ev| set_link_kp.set(event_target_value(&ev))
                />
                <button class="sv-btn-primary" style="margin-top:14px;width:100%;" on:click=link_action>
                    "Link"
                </button>
                {move || status.get().map(|s| view! {
                    <div style="margin-top:12px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
            </div>
        </div>
    }
}
