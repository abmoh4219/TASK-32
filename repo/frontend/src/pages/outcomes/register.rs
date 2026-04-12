//! Multi-step register-outcome form: type → details → contributors → submit.
//! Step 2 also surfaces duplicate-detection candidates from the create endpoint.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::outcomes::{
    self as out_api, AddContributorInput, CompareResult, CreateOutcomeInput, DuplicateCandidate,
};
use crate::logic::validation::{share_total_color, share_total_state, ShareTotalState};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Step {
    Type,
    Details,
    Contributors,
    Done,
}

#[component]
pub fn RegisterOutcome() -> impl IntoView {
    let (step, set_step) = create_signal(Step::Type);
    let (otype, set_otype) = create_signal::<String>("paper".to_string());
    let (title, set_title) = create_signal(String::new());
    let (abstract_text, set_abstract) = create_signal(String::new());
    let (cert, set_cert) = create_signal(String::new());

    let (created_id, set_created_id) = create_signal::<Option<String>>(None);
    let (duplicates, set_duplicates) = create_signal::<Vec<DuplicateCandidate>>(Vec::new());
    let (status, set_status) = create_signal::<Option<String>>(None);

    let (contributors, set_contributors) = create_signal::<Vec<(String, i64)>>(Vec::new());
    let (new_contrib_user, set_new_contrib_user) = create_signal::<String>("u-reviewer".into());
    let (new_contrib_share, set_new_contrib_share) = create_signal::<i64>(100);
    // (filename, file_size_bytes) — simple tuple so the signal type compiles
    // on both WASM and native targets (the actual upload only runs in-browser).
    let (evidence_files, set_evidence_files) = create_signal::<Vec<(String, i64)>>(Vec::new());
    // Duplicate-gating: user must acknowledge/compare duplicates before submit.
    let (dup_acknowledged, set_dup_acknowledged) = create_signal(false);
    // Inline compare result — shown when user clicks "Compare" on a duplicate.
    let (inline_compare, set_inline_compare) = create_signal::<Option<CompareResult>>(None);

    let create_action = move |_| {
        let payload = CreateOutcomeInput {
            r#type: otype.get(),
            title: title.get(),
            abstract_snippet: abstract_text.get(),
            certificate_number: if cert.get().is_empty() { None } else { Some(cert.get()) },
        };
        spawn_local(async move {
            match out_api::create_outcome(payload).await {
                Ok(res) => {
                    set_created_id.set(Some(res.outcome.id.clone()));
                    set_duplicates.set(res.duplicate_candidates);
                    set_step.set(Step::Contributors);
                    set_status.set(Some(format!("Outcome draft created: {}", res.outcome.id)));
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let add_contributor = move |_| {
        let id = match created_id.get() {
            Some(v) => v,
            None => {
                set_status.set(Some("Create the outcome first".into()));
                return;
            }
        };
        let user = new_contrib_user.get();
        let share = new_contrib_share.get();
        spawn_local(async move {
            let res = out_api::add_contributor(
                &id,
                AddContributorInput {
                    user_id: user.clone(),
                    share_percentage: share,
                    role_in_work: None,
                },
            )
            .await;
            match res {
                Ok(_) => {
                    set_contributors.update(|v| v.push((user, share)));
                    set_status.set(Some("Contributor added".into()));
                }
                Err(e) => set_status.set(Some(format!("Rejected: {}", e.message))),
            }
        });
    };

    let submit_action = move |_| {
        let id = match created_id.get() {
            Some(v) => v,
            None => return,
        };
        spawn_local(async move {
            match out_api::submit_outcome(&id).await {
                Ok(_) => {
                    set_status.set(Some("Submitted for review".into()));
                    set_step.set(Step::Done);
                }
                Err(e) => set_status.set(Some(format!("Submit failed: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1fr 320px;gap:24px;">
            <div class="sv-card">
                {move || match step.get() {
                    Step::Type => view! {
                        <h2 style="margin:0 0 16px;font-size:18px;color:#F5C518;">"Step 1 — Type"</h2>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:12px;">
                            {type_card("paper", "Research Paper", "Peer-reviewed publication", otype, set_otype)}
                            {type_card("patent", "Patent", "Filed or granted patent", otype, set_otype)}
                            {type_card("competition_result", "Competition Result", "Award or placement", otype, set_otype)}
                            {type_card("software_copyright", "Software Copyright", "Registered software work", otype, set_otype)}
                        </div>
                        <button class="sv-btn-primary" style="margin-top:24px;" on:click=move |_| set_step.set(Step::Details)>
                            "Next →"
                        </button>
                    }.into_view(),
                    Step::Details => view! {
                        <h2 style="margin:0 0 16px;font-size:18px;color:#F5C518;">"Step 2 — Details"</h2>
                        <label class="sv-label">"Title"</label>
                        <input class="sv-input" prop:value=move || title.get()
                            on:input=move |ev| set_title.set(event_target_value(&ev))/>
                        <label class="sv-label" style="margin-top:14px;">"Abstract / description"</label>
                        <textarea class="sv-input" rows="4" prop:value=move || abstract_text.get()
                            on:input=move |ev| set_abstract.set(event_target_value(&ev))></textarea>
                        <label class="sv-label" style="margin-top:14px;">"Certificate number (optional)"</label>
                        <input class="sv-input" prop:value=move || cert.get()
                            on:input=move |ev| set_cert.set(event_target_value(&ev))/>
                        <div style="display:flex;gap:8px;margin-top:24px;">
                            <button class="sv-btn-secondary" on:click=move |_| set_step.set(Step::Type)>"← Back"</button>
                            <button class="sv-btn-primary" on:click=create_action>"Create draft & check duplicates →"</button>
                        </div>
                    }.into_view(),
                    Step::Contributors => view! {
                        <h2 style="margin:0 0 16px;font-size:18px;color:#F5C518;">"Step 3 — Contributors"</h2>
                        {move || {
                            let dups = duplicates.get();
                            if dups.is_empty() {
                                // No duplicates — auto-acknowledge so submit is enabled.
                                set_dup_acknowledged.set(true);
                                ().into_view()
                            } else {
                                let current_id = created_id.get().unwrap_or_default();
                                view! {
                                    <div class="sv-card" style="background:rgba(245,158,11,0.10);border-color:rgba(245,158,11,0.40);margin-bottom:16px;">
                                        <div style="font-size:12px;color:#F59E0B;font-weight:600;margin-bottom:6px;">
                                            {format!("⚠ {} similar outcome(s) found — review before submitting", dups.len())}
                                        </div>
                                        {dups.into_iter().take(5).map(|d| {
                                            let cur = current_id.clone();
                                            let dup_id = d.id.clone();
                                            view! {
                                                <div style="display:flex;justify-content:space-between;align-items:center;font-size:12px;color:#A0A0B0;padding:4px 0;">
                                                    <span>{format!("• {} ({}, score {:.2})", d.title, d.reason, d.similarity_score)}</span>
                                                    <button
                                                        class="sv-btn-secondary" style="font-size:10px;padding:3px 8px;"
                                                        on:click=move |_| {
                                                            let a = cur.clone();
                                                            let b = dup_id.clone();
                                                            spawn_local(async move {
                                                                match out_api::compare_outcomes(&a, &b).await {
                                                                    Ok(r) => set_inline_compare.set(Some(r)),
                                                                    Err(e) => set_status.set(Some(format!("Compare error: {}", e.message))),
                                                                }
                                                            });
                                                        }
                                                    >
                                                        "Compare"
                                                    </button>
                                                </div>
                                            }
                                        }).collect_view()}
                                        <div style="margin-top:10px;border-top:1px solid rgba(245,158,11,0.20);padding-top:10px;">
                                            <label style="display:flex;align-items:center;gap:8px;font-size:12px;color:#F59E0B;cursor:pointer;">
                                                <input type="checkbox"
                                                    prop:checked=move || dup_acknowledged.get()
                                                    on:change=move |ev| {
                                                        let checked = event_target_checked(&ev);
                                                        set_dup_acknowledged.set(checked);
                                                    }/>
                                                "I have reviewed the duplicates and confirm this is a new outcome"
                                            </label>
                                        </div>
                                    </div>
                                }.into_view()
                            }
                        }}
                        // ── Inline side-by-side compare panel ──
                        {move || inline_compare.get().map(|r| {
                            let title_diff = r.a.title != r.b.title;
                            let abstract_diff = r.a.abstract_snippet != r.b.abstract_snippet;
                            let cert_diff = r.a.certificate_number != r.b.certificate_number;
                            view! {
                                <div class="sv-card" style="margin-bottom:16px;border-color:rgba(245,197,24,0.30);">
                                    <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Side-by-Side Compare"</h3>
                                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;font-size:12px;">
                                        <div>
                                            <div style="font-weight:600;color:#F5C518;margin-bottom:6px;">"Current Draft"</div>
                                            <div style="color:#A0A0B0;font-family:monospace;font-size:10px;margin-bottom:4px;">{r.a.id.clone()}</div>
                                            <div style=if title_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;" } else { "padding:4px;" }>
                                                <span style="color:#A0A0B0;">"Title: "</span>{r.a.title.clone()}
                                            </div>
                                            <div style=if abstract_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;margin-top:4px;" } else { "padding:4px;margin-top:4px;" }>
                                                <span style="color:#A0A0B0;">"Abstract: "</span>{r.a.abstract_snippet.clone()}
                                            </div>
                                            <div style=if cert_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;margin-top:4px;" } else { "padding:4px;margin-top:4px;" }>
                                                <span style="color:#A0A0B0;">"Certificate: "</span>{r.a.certificate_number.clone().unwrap_or_else(|| "—".into())}
                                            </div>
                                        </div>
                                        <div>
                                            <div style="font-weight:600;color:#F5C518;margin-bottom:6px;">"Existing Match"</div>
                                            <div style="color:#A0A0B0;font-family:monospace;font-size:10px;margin-bottom:4px;">{r.b.id.clone()}</div>
                                            <div style=if title_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;" } else { "padding:4px;" }>
                                                <span style="color:#A0A0B0;">"Title: "</span>{r.b.title.clone()}
                                            </div>
                                            <div style=if abstract_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;margin-top:4px;" } else { "padding:4px;margin-top:4px;" }>
                                                <span style="color:#A0A0B0;">"Abstract: "</span>{r.b.abstract_snippet.clone()}
                                            </div>
                                            <div style=if cert_diff { "background:rgba(245,158,11,0.10);padding:4px;border-radius:3px;margin-top:4px;" } else { "padding:4px;margin-top:4px;" }>
                                                <span style="color:#A0A0B0;">"Certificate: "</span>{r.b.certificate_number.clone().unwrap_or_else(|| "—".into())}
                                            </div>
                                        </div>
                                    </div>
                                    <div style="margin-top:10px;font-size:12px;color:#F5C518;font-weight:600;">
                                        {format!("Similarity → title {:.2}, abstract {:.2}", r.title_similarity, r.abstract_similarity)}
                                    </div>
                                </div>
                            }
                        })}
                        <div style="display:grid;grid-template-columns:1fr 100px auto;gap:8px;align-items:end;">
                            <div>
                                <label class="sv-label">"User id"</label>
                                <input class="sv-input" prop:value=move || new_contrib_user.get()
                                    on:input=move |ev| set_new_contrib_user.set(event_target_value(&ev))/>
                            </div>
                            <div>
                                <label class="sv-label">"Share %"</label>
                                <input class="sv-input" type="number" min="1" max="100"
                                    prop:value=move || new_contrib_share.get().to_string()
                                    on:input=move |ev| set_new_contrib_share.set(event_target_value(&ev).parse().unwrap_or(0))/>
                            </div>
                            <button class="sv-btn-secondary" on:click=add_contributor>"Add"</button>
                        </div>

                        <div style="margin-top:18px;">
                            <table class="sv-table">
                                <thead><tr><th>"User"</th><th>"Share"</th></tr></thead>
                                <tbody>
                                    {move || contributors.get().into_iter().map(|(u, s)| view! {
                                        <tr><td>{u}</td><td>{format!("{}%", s)}</td></tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                        </div>

                        {move || {
                            let shares: Vec<i64> = contributors.get().iter().map(|(_, s)| *s).collect();
                            let total: i64 = shares.iter().sum();
                            let state = share_total_state(&shares);
                            let label = match state {
                                ShareTotalState::Complete => "Ready to submit (100%)",
                                ShareTotalState::Under => "Under-allocated",
                                ShareTotalState::Over => "Over-allocated",
                            };
                            view! {
                                <div style=move || format!("margin-top:14px;padding:10px 14px;border-radius:6px;font-size:12px;font-weight:600;color:{};background:rgba(255,255,255,0.04);", share_total_color(&shares))>
                                    {format!("{} — total {}%", label, total)}
                                </div>
                            }
                        }}

                        // ── Evidence upload ──
                        <div style="margin-top:24px;border-top:1px solid rgba(245,197,24,0.10);padding-top:16px;">
                            <h3 style="margin:0 0 8px;font-size:13px;color:#F5C518;">"Evidence files (PDF/JPG/PNG)"</h3>
                            <input
                                type="file"
                                accept=".pdf,.jpg,.jpeg,.png"
                                class="sv-input"
                                style="padding:6px;"
                                on:change=move |ev| {
                                    use wasm_bindgen::JsCast;
                                    let input: web_sys::HtmlInputElement =
                                        ev.target().unwrap().unchecked_into();
                                    if let Some(files) = input.files() {
                                        if let Some(file) = files.get(0) {
                                            let oid = match created_id.get() {
                                                Some(v) => v,
                                                None => {
                                                    set_status.set(Some("Create outcome first".into()));
                                                    return;
                                                }
                                            };
                                            #[cfg(target_arch = "wasm32")]
                                {
                                    spawn_local(async move {
                                        match out_api::upload_evidence(&oid, file).await {
                                            Ok(ef) => {
                                                set_evidence_files.update(|v| v.push((ef.filename, ef.file_size)));
                                                set_status.set(Some("Evidence uploaded".into()));
                                            }
                                            Err(e) => set_status.set(Some(format!("Upload error: {}", e.message))),
                                        }
                                    });
                                }
                                #[cfg(not(target_arch = "wasm32"))]
                                {
                                    let _ = (oid, file, set_evidence_files);
                                    set_status.set(Some("Upload only available in browser".into()));
                                }
                                        }
                                    }
                                }
                            />
                            {move || {
                                let files = evidence_files.get();
                                if files.is_empty() { ().into_view() } else {
                                    view! {
                                        <div style="margin-top:8px;font-size:11px;color:#A0A0B0;">
                                            {files.into_iter().map(|(name, size)| view! {
                                                <div>{format!("✓ {} ({} KB)", name, size / 1024)}</div>
                                            }).collect_view()}
                                        </div>
                                    }.into_view()
                                }
                            }}
                        </div>

                        <div style="display:flex;gap:8px;margin-top:24px;">
                            <button class="sv-btn-secondary" on:click=move |_| set_step.set(Step::Details)>"← Back"</button>
                            {move || {
                                let enabled = dup_acknowledged.get();
                                view! {
                                    <button
                                        class="sv-btn-primary"
                                        prop:disabled=move || !enabled
                                        style=move || if enabled { "" } else { "opacity:0.5;cursor:not-allowed;" }
                                        on:click=submit_action
                                    >
                                        {if enabled { "Submit for review" } else { "Acknowledge duplicates first" }}
                                    </button>
                                }
                            }}
                        </div>
                    }.into_view(),
                    Step::Done => view! {
                        <h2 class="sv-text-gradient" style="margin:0 0 12px;font-size:22px;">"Submitted ✓"</h2>
                        <p style="color:#A0A0B0;">"This outcome is now in the review queue."</p>
                        <button class="sv-btn-secondary" on:click=move |_| set_step.set(Step::Type)>"Register another"</button>
                    }.into_view(),
                }}
            </div>

            <aside class="sv-card">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Progress"</h3>
                <div style="display:flex;flex-direction:column;gap:8px;font-size:12px;">
                    {step_indicator("1. Type", Step::Type, step)}
                    {step_indicator("2. Details", Step::Details, step)}
                    {step_indicator("3. Contributors", Step::Contributors, step)}
                    {step_indicator("4. Submitted", Step::Done, step)}
                </div>
                {move || status.get().map(|s| view! {
                    <div style="margin-top:14px;padding:10px;background:rgba(96,165,250,0.05);border-radius:6px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
            </aside>
        </div>
    }
}

fn type_card(
    value: &'static str,
    title: &'static str,
    description: &'static str,
    otype: ReadSignal<String>,
    setter: WriteSignal<String>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| setter.set(value.to_string())
            style=move || {
                let active = otype.get() == value;
                format!(
                    "padding:18px;border-radius:8px;cursor:pointer;text-align:left;background:{};border:1px solid {};color:#F0F0F5;",
                    if active { "rgba(245,197,24,0.10)" } else { "rgba(255,255,255,0.02)" },
                    if active { "rgba(245,197,24,0.50)" } else { "rgba(255,255,255,0.08)" }
                )
            }
        >
            <div style="font-weight:600;font-size:13px;color:#F5C518;">{title}</div>
            <div style="margin-top:4px;font-size:11px;color:#A0A0B0;">{description}</div>
        </button>
    }
}

fn step_indicator(label: &'static str, this_step: Step, current: ReadSignal<Step>) -> impl IntoView {
    view! {
        <div style=move || {
            let active = current.get() == this_step;
            format!("padding:6px 10px;border-left:3px solid {};color:{};", if active { "#F5C518" } else { "rgba(255,255,255,0.10)" }, if active { "#F5C518" } else { "#A0A0B0" })
        }>
            {label}
        </div>
    }
}
