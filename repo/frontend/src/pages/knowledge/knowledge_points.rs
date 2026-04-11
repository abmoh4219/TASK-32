//! Knowledge points tab — filterable table backed by `/api/knowledge/points`
//! plus a bulk-edit modal that calls the preview + apply endpoints.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::knowledge::{
    self as kn_api, BulkUpdate, BulkUpdateRequest, ConflictPreview, CreateKnowledgePointInput,
    KnowledgeFilter, KnowledgePoint,
};
use crate::logic::filter::{DiscriminationBandPreset, KnowledgeFilterState};

#[component]
pub fn KnowledgePointsTab() -> impl IntoView {
    let (filter, set_filter) = create_signal(KnowledgeFilterState::new());

    let kps = create_resource(move || filter.get(), |f| async move {
        let api_filter = KnowledgeFilter {
            category_id: None,
            difficulty_min: f.difficulty_min,
            difficulty_max: f.difficulty_max,
            discrimination_min: f.discrimination_min,
            discrimination_max: f.discrimination_max,
            tags: f.tags.clone(),
            chapter: f.chapter.clone(),
        };
        kn_api::list_knowledge_points(Some(api_filter)).await
    });

    let (selected_ids, set_selected_ids) = create_signal::<Vec<String>>(Vec::new());
    let (bulk_difficulty, set_bulk_difficulty) = create_signal::<Option<i64>>(None);
    let (preview, set_preview) = create_signal::<Vec<ConflictPreview>>(Vec::new());
    let (status, set_status) = create_signal::<Option<String>>(None);

    let (new_title, set_new_title) = create_signal(String::new());
    let (new_category, set_new_category) = create_signal(String::new());
    let (new_difficulty, set_new_difficulty) = create_signal(3i64);
    let (new_disc, set_new_disc) = create_signal(0.3f64);

    let create_kp = move |_| {
        let title = new_title.get();
        let category = new_category.get();
        if title.trim().is_empty() || category.trim().is_empty() {
            set_status.set(Some("Title and category id required".into()));
            return;
        }
        let diff = new_difficulty.get();
        let disc = new_disc.get();
        spawn_local(async move {
            let res = kn_api::create_knowledge_point(CreateKnowledgePointInput {
                category_id: category,
                title,
                content: String::new(),
                difficulty: diff,
                discrimination: disc,
                tags: Vec::new(),
            })
            .await;
            match res {
                Ok(_) => {
                    set_status.set(Some("Knowledge point created".into()));
                    set_new_title.set(String::new());
                    kps.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let preview_action = move |_| {
        let ids = selected_ids.get();
        if ids.len() > 1000 {
            set_status.set(Some("Bulk edit limited to 1000 records".into()));
            return;
        }
        let target = bulk_difficulty.get();
        if target.is_none() {
            set_status.set(Some("Pick a target difficulty first".into()));
            return;
        }
        let req = BulkUpdateRequest {
            ids,
            changes: BulkUpdate {
                difficulty: target,
                ..Default::default()
            },
        };
        spawn_local(async move {
            match kn_api::bulk_preview(req).await {
                Ok(p) => {
                    set_preview.set(p);
                    set_status.set(Some("Preview loaded".into()));
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let apply_action = move |_| {
        let ids = selected_ids.get();
        let target = bulk_difficulty.get();
        if ids.is_empty() || target.is_none() {
            set_status.set(Some("Select rows + difficulty first".into()));
            return;
        }
        let req = BulkUpdateRequest {
            ids,
            changes: BulkUpdate {
                difficulty: target,
                ..Default::default()
            },
        };
        spawn_local(async move {
            match kn_api::bulk_apply(req).await {
                Ok(_) => {
                    set_status.set(Some("Bulk update applied".into()));
                    set_preview.set(Vec::new());
                    kps.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:280px 1fr;gap:24px;">
            <aside class="sv-card">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Filters"</h3>
                <label class="sv-label">"Difficulty range"</label>
                <div style="display:flex;gap:6px;">
                    <input
                        class="sv-input" type="number" min="1" max="5" placeholder="min"
                        on:input=move |ev| {
                            let v = event_target_value(&ev).parse::<i64>().ok();
                            set_filter.update(|f| f.difficulty_min = v);
                        }
                    />
                    <input
                        class="sv-input" type="number" min="1" max="5" placeholder="max"
                        on:input=move |ev| {
                            let v = event_target_value(&ev).parse::<i64>().ok();
                            set_filter.update(|f| f.difficulty_max = v);
                        }
                    />
                </div>

                <label class="sv-label" style="margin-top:14px;">"Discrimination band"</label>
                <div style="display:flex;flex-wrap:wrap;gap:6px;">
                    {band_btn("Poor", DiscriminationBandPreset::Poor, set_filter)}
                    {band_btn("Acceptable", DiscriminationBandPreset::Acceptable, set_filter)}
                    {band_btn("Good", DiscriminationBandPreset::Good, set_filter)}
                    {band_btn("Excellent", DiscriminationBandPreset::Excellent, set_filter)}
                </div>

                <button
                    class="sv-btn-ghost"
                    style="margin-top:18px;width:100%;"
                    on:click=move |_| set_filter.update(|f| f.clear())
                >
                    "Clear filters"
                </button>

                <div style="margin-top:24px;border-top:1px solid rgba(245,197,24,0.10);padding-top:18px;">
                    <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"New Knowledge Point"</h3>
                    <label class="sv-label">"Title"</label>
                    <input class="sv-input" prop:value=move || new_title.get()
                        on:input=move |ev| set_new_title.set(event_target_value(&ev))/>
                    <label class="sv-label" style="margin-top:8px;">"Category id"</label>
                    <input class="sv-input" placeholder="cat-algebra" prop:value=move || new_category.get()
                        on:input=move |ev| set_new_category.set(event_target_value(&ev))/>
                    <label class="sv-label" style="margin-top:8px;">"Difficulty"</label>
                    <input class="sv-input" type="number" min="1" max="5"
                        prop:value=move || new_difficulty.get().to_string()
                        on:input=move |ev| set_new_difficulty.set(event_target_value(&ev).parse().unwrap_or(3))/>
                    <label class="sv-label" style="margin-top:8px;">"Discrimination"</label>
                    <input class="sv-input" type="number" step="0.05" min="-1" max="1"
                        prop:value=move || new_disc.get().to_string()
                        on:input=move |ev| set_new_disc.set(event_target_value(&ev).parse().unwrap_or(0.3))/>
                    <button class="sv-btn-primary" style="margin-top:14px;width:100%;" on:click=create_kp>
                        "Create"
                    </button>
                </div>
            </aside>

            <div>
                <div class="sv-card" style="margin-bottom:16px;display:flex;align-items:center;gap:12px;flex-wrap:wrap;">
                    <span style="font-size:12px;color:#A0A0B0;">"Bulk edit (max 1,000 rows):"</span>
                    <input
                        class="sv-input" type="number" min="1" max="5" placeholder="target difficulty"
                        style="width:160px;"
                        on:input=move |ev| set_bulk_difficulty.set(event_target_value(&ev).parse().ok())
                    />
                    <button class="sv-btn-secondary" on:click=preview_action>"Preview conflicts"</button>
                    <button class="sv-btn-primary" on:click=apply_action>"Apply"</button>
                    <span style="color:#A0A0B0;font-size:11px;">
                        {move || format!("{} selected", selected_ids.get().len())}
                    </span>
                </div>

                {move || status.get().map(|msg| view! {
                    <div class="sv-card" style="margin-bottom:12px;background:rgba(96,165,250,0.05);font-size:12px;">{msg}</div>
                })}

                {move || {
                    let p = preview.get();
                    if p.is_empty() { ().into_view() } else {
                        view! {
                            <div class="sv-card" style="margin-bottom:12px;background:rgba(245,158,11,0.05);">
                                <div style="font-size:12px;color:#F59E0B;margin-bottom:6px;font-weight:600;">
                                    {format!("{} field(s) would change", p.len())}
                                </div>
                                <div style="font-size:11px;color:#A0A0B0;">
                                    {p.into_iter().take(8).map(|c| view! {
                                        <div>{format!("{} → {}: {} → {}", c.kp_id, c.field, c.current_value, c.proposed_value)}</div>
                                    }).collect_view()}
                                </div>
                            </div>
                        }.into_view()
                    }
                }}

                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:200px;"></div> }>
                    {move || kps.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div class="sv-card" style="text-align:center;color:#A0A0B0;padding:40px;">
                                "No knowledge points match the current filters."
                            </div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead>
                                    <tr>
                                        <th style="width:32px;"></th>
                                        <th>"Title"</th>
                                        <th>"Category"</th>
                                        <th>"Difficulty"</th>
                                        <th>"Discrimination"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|kp| render_row(kp, set_selected_ids)).collect_view()}
                                </tbody>
                            </table>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{format!("Failed: {}", e.message)}</div> }.into_view(),
                    })}
                </Suspense>
            </div>
        </div>
    }
}

fn render_row(kp: KnowledgePoint, set_selected: WriteSignal<Vec<String>>) -> impl IntoView {
    let id_for_check = kp.id.clone();
    view! {
        <tr>
            <td>
                <input
                    type="checkbox"
                    on:change=move |ev| {
                        let checked = event_target_checked(&ev);
                        let id = id_for_check.clone();
                        set_selected.update(|v| {
                            if checked && !v.contains(&id) { v.push(id); }
                            else { v.retain(|x| x != &id_for_check); }
                        });
                    }
                />
            </td>
            <td style="font-weight:500;">{kp.title}</td>
            <td style="font-family:monospace;font-size:11px;color:#A0A0B0;">{kp.category_id}</td>
            <td>
                <span class="sv-badge sv-badge-info">{kp.difficulty.to_string()}</span>
            </td>
            <td>{format!("{:.2}", kp.discrimination)}</td>
        </tr>
    }
}

fn band_btn(
    label: &'static str,
    band: DiscriminationBandPreset,
    set_filter: WriteSignal<KnowledgeFilterState>,
) -> impl IntoView {
    view! {
        <button
            class="sv-btn-ghost"
            style="font-size:11px;padding:6px 10px;"
            on:click=move |_| set_filter.update(|f| f.apply_discrimination_band(band))
        >
            {label}
        </button>
    }
}
