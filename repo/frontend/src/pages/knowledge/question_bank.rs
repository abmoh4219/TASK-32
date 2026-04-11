//! Question bank tab — list questions and their linked knowledge points.

use leptos::*;

use crate::api::knowledge as kn_api;

#[component]
pub fn QuestionBankTab() -> impl IntoView {
    let questions = create_resource(|| (), |_| async move { kn_api::list_questions().await });

    view! {
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
    }
}
