//! Category tree tab — renders a recursive DAG view, supports creating new
//! categories, viewing reference counts, and merging.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::knowledge::{
    self as kn_api, CategoryNode, CreateCategoryInput, MergeRequest, ReferenceCount,
};

#[component]
pub fn CategoryTreeTab() -> impl IntoView {
    let tree = create_resource(|| (), |_| async move { kn_api::get_category_tree().await });
    let (new_name, set_new_name) = create_signal(String::new());
    let (selected_parent, set_selected_parent) = create_signal::<Option<String>>(None);
    let (status, set_status) = create_signal::<Option<String>>(None);
    let (selected_for_ref, set_selected_for_ref) = create_signal::<Option<String>>(None);
    let (ref_count, set_ref_count) = create_signal::<Option<ReferenceCount>>(None);
    let (merge_source, set_merge_source) = create_signal(String::new());
    let (merge_target, set_merge_target) = create_signal(String::new());

    let create_action = move |_| {
        let name = new_name.get();
        if name.trim().is_empty() {
            set_status.set(Some("Name is required".to_string()));
            return;
        }
        let parent = selected_parent.get();
        spawn_local(async move {
            let res = kn_api::create_category(CreateCategoryInput {
                name,
                parent_id: parent,
                description: None,
            })
            .await;
            match res {
                Ok(c) => {
                    set_status.set(Some(format!("Created \"{}\"", c.name)));
                    set_new_name.set(String::new());
                    tree.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let view_refs = move |id: String| {
        set_selected_for_ref.set(Some(id.clone()));
        spawn_local(async move {
            match kn_api::get_reference_count(&id).await {
                Ok(rc) => set_ref_count.set(Some(rc)),
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let merge_action = move |_| {
        let s = merge_source.get();
        let t = merge_target.get();
        if s.is_empty() || t.is_empty() {
            set_status.set(Some("Pick both source and target ids".into()));
            return;
        }
        spawn_local(async move {
            let res = kn_api::merge_categories(MergeRequest {
                source_id: s.clone(),
                target_id: t.clone(),
            })
            .await;
            match res {
                Ok(_) => {
                    set_status.set(Some(format!("Merged {} → {}", s, t)));
                    tree.refetch();
                }
                Err(e) => set_status.set(Some(format!("Merge blocked: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1.4fr 1fr;gap:20px;">
            <div class="sv-card">
                <div class="sv-card-header">
                    <div>
                        <div class="sv-card-title">"Category Tree"</div>
                        <div class="sv-card-subtitle">"Hierarchical DAG with live reference counts"</div>
                    </div>
                </div>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:120px;width:100%;"></div> }>
                    {move || tree.get().map(|res| match res {
                        Ok(nodes) if nodes.is_empty() => view! {
                            <div style="color:#A0A0B0;font-size:13px;padding:12px 0;">"No categories yet — create one →"</div>
                        }.into_view(),
                        Ok(nodes) => view! {
                            <ul style="list-style:none;padding:0;margin:0;">
                                {nodes.into_iter().map(|n| render_node(n, view_refs.clone())).collect_view()}
                            </ul>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{format!("Failed to load: {}", e.message)}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div style="display:flex;flex-direction:column;gap:16px;">
                <div class="sv-card">
                    <div class="sv-card-header"><div class="sv-card-title">"Create Category"</div></div>
                    <label class="sv-label">"Name"</label>
                    <input
                        class="sv-input"
                        prop:value=move || new_name.get()
                        on:input=move |ev| set_new_name.set(event_target_value(&ev))
                    />
                    <label class="sv-label" style="margin-top:12px;">"Parent ID (optional)"</label>
                    <input
                        class="sv-input"
                        placeholder="e.g. cat-mathematics"
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            set_selected_parent.set(if v.is_empty() { None } else { Some(v) });
                        }
                    />
                    <button class="sv-btn-primary" style="margin-top:16px;width:100%;" on:click=create_action>
                        "Create Category"
                    </button>
                </div>

                <div class="sv-card">
                    <div class="sv-card-header"><div class="sv-card-title">"Merge Categories"</div></div>
                    <label class="sv-label">"Source ID"</label>
                    <input
                        class="sv-input"
                        prop:value=move || merge_source.get()
                        on:input=move |ev| set_merge_source.set(event_target_value(&ev))
                    />
                    <label class="sv-label" style="margin-top:12px;">"Target ID"</label>
                    <input
                        class="sv-input"
                        prop:value=move || merge_target.get()
                        on:input=move |ev| set_merge_target.set(event_target_value(&ev))
                    />
                    <button class="sv-btn-primary" style="margin-top:16px;width:100%;" on:click=merge_action>
                        "Merge Source → Target"
                    </button>
                    <p style="font-size:11px;color:#A0A0B0;margin-top:8px;">
                        "Blocked if it would create a cycle in the DAG."
                    </p>
                </div>

                {move || ref_count.get().map(|rc| {
                    let id = selected_for_ref.get().unwrap_or_default();
                    view! {
                        <div class="sv-card" style="background:rgba(245,197,24,0.05);">
                            <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">{format!("References → {}", id)}</h3>
                            <div style="font-size:13px;line-height:1.8;">
                                <div>{format!("Direct knowledge points: {}", rc.direct_kp_count)}</div>
                                <div>{format!("Child categories: {}", rc.child_category_count)}</div>
                                <div>{format!("Linked questions: {}", rc.indirect_question_count)}</div>
                                <div style="margin-top:8px;font-weight:600;color:#F5C518;">
                                    {format!("Total: {}", rc.total)}
                                </div>
                            </div>
                        </div>
                    }
                })}

                {move || status.get().map(|msg| view! {
                    <div class="sv-card" style="background:rgba(96,165,250,0.05);font-size:12px;">{msg}</div>
                })}
            </div>
        </div>
    }
}

fn render_node<F: Fn(String) + Clone + 'static>(node: CategoryNode, view_refs: F) -> View {
    let id = node.category.id.clone();
    let label = format!(
        "{} ({} kp{})",
        node.category.name,
        node.kp_count,
        if node.kp_count == 1 { "" } else { "s" }
    );
    let view_refs_for_btn = view_refs.clone();
    let id_for_btn = id.clone();
    let children = node.children;
    let view_refs_for_children = view_refs.clone();
    view! {
        <li style="margin:6px 0;">
            <div style="display:flex;align-items:center;gap:10px;">
                <span style="color:#F5C518;">"●"</span>
                <span style="font-size:13px;">{label}</span>
                <span style="color:#A0A0B0;font-size:11px;font-family:monospace;">{id.clone()}</span>
                <button
                    class="sv-btn-ghost"
                    style="padding:4px 10px;font-size:11px;"
                    on:click=move |_| view_refs_for_btn(id_for_btn.clone())
                >
                    "refs"
                </button>
            </div>
            {if !children.is_empty() {
                view! {
                    <ul style="list-style:none;padding:0 0 0 20px;border-left:1px solid rgba(245,197,24,0.15);margin:4px 0 4px 6px;">
                        {children.into_iter().map(|c| render_node(c, view_refs_for_children.clone())).collect_view()}
                    </ul>
                }.into_view()
            } else {
                ().into_view()
            }}
        </li>
    }.into_view()
}
