//! Knowledge module Leptos pages — category tree, knowledge points, question bank.
//! `KnowledgePage` is the routed entry; it lays out a tab strip and delegates
//! the body to the corresponding submodule component.

use leptos::*;

use crate::components::layout::{NavTarget, PageShell};

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
        <PageShell
            active=NavTarget::Knowledge
            title="Knowledge Management"
            subtitle="Curate the category tree, knowledge points, and question bank"
        >
            <div class="sv-page-header">
                <h1 class="sv-page-title">"Knowledge Management"</h1>
                <p class="sv-page-subtitle">"Curate the DAG, manage knowledge points, and link questions to the bank."</p>
            </div>

            <div class="sv-tabs">
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
        </PageShell>
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
            class=move || if current.get() == this_tab { "sv-tab active" } else { "sv-tab" }
            on:click=move |_| setter.set(this_tab)
        >
            {label}
        </button>
    }
}
