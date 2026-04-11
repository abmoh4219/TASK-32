//! Knowledge module Leptos pages — category tree, knowledge points, question bank.
//! `KnowledgePage` is the routed entry; it lays out a tab strip and delegates
//! the body to the corresponding submodule component.

use leptos::*;

pub mod category_tree;
pub mod knowledge_points;
pub mod question_bank;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeTab {
    Tree,
    Points,
    Questions,
}

#[component]
pub fn KnowledgePage() -> impl IntoView {
    let (tab, set_tab) = create_signal(KnowledgeTab::Tree);

    view! {
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">
                        "Knowledge Management"
                    </h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">
                        "Curate the category tree, knowledge points, and question bank."
                    </p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            <div style="display:flex;gap:8px;border-bottom:1px solid rgba(245,197,24,0.20);margin-bottom:24px;">
                {tab_button("Category Tree", KnowledgeTab::Tree, tab, set_tab)}
                {tab_button("Knowledge Points", KnowledgeTab::Points, tab, set_tab)}
                {tab_button("Question Bank", KnowledgeTab::Questions, tab, set_tab)}
            </div>

            <div>
                {move || match tab.get() {
                    KnowledgeTab::Tree => view! { <category_tree::CategoryTreeTab/> }.into_view(),
                    KnowledgeTab::Points => view! { <knowledge_points::KnowledgePointsTab/> }.into_view(),
                    KnowledgeTab::Questions => view! { <question_bank::QuestionBankTab/> }.into_view(),
                }}
            </div>
        </div>
    }
}

fn tab_button(
    label: &'static str,
    this_tab: KnowledgeTab,
    current: ReadSignal<KnowledgeTab>,
    setter: WriteSignal<KnowledgeTab>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| setter.set(this_tab)
            style=move || {
                let is_active = current.get() == this_tab;
                format!(
                    "padding:12px 20px;background:transparent;border:none;cursor:pointer;font-size:13px;font-weight:600;color:{};border-bottom:2px solid {};margin-bottom:-1px;transition:all 0.15s;",
                    if is_active { "#F5C518" } else { "#A0A0B0" },
                    if is_active { "#F5C518" } else { "transparent" }
                )
            }
        >
            {label}
        </button>
    }
}
