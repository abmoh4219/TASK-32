//! Store / promotions Leptos pages.

use leptos::*;

pub mod promotions;
pub mod checkout;

#[derive(Clone, Copy, PartialEq, Eq)]
enum StoreTab {
    Promotions,
    Checkout,
    Orders,
}

#[component]
pub fn StorePage() -> impl IntoView {
    let (tab, set_tab) = create_signal(StoreTab::Promotions);

    view! {
        <div style="min-height:100vh;padding:32px 40px;max-width:1280px;margin:0 auto;">
            <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:24px;">
                <div>
                    <h1 class="sv-text-gradient" style="font-size:28px;font-weight:800;margin:0;">"Store & Promotions"</h1>
                    <p style="color:#A0A0B0;margin:6px 0 0;font-size:13px;">"Configure offline promotions, run checkout with the best-offer engine."</p>
                </div>
                <a href="/" class="sv-btn-ghost">"Logout"</a>
            </div>

            <div style="display:flex;gap:8px;border-bottom:1px solid rgba(245,197,24,0.20);margin-bottom:24px;">
                {tab_btn("Promotions", StoreTab::Promotions, tab, set_tab)}
                {tab_btn("Checkout", StoreTab::Checkout, tab, set_tab)}
                {tab_btn("Orders", StoreTab::Orders, tab, set_tab)}
            </div>

            {move || match tab.get() {
                StoreTab::Promotions => view! { <promotions::PromotionsTab /> }.into_view(),
                StoreTab::Checkout => view! { <checkout::CheckoutTab /> }.into_view(),
                StoreTab::Orders => view! { <OrdersList /> }.into_view(),
            }}
        </div>
    }
}

#[component]
fn OrdersList() -> impl IntoView {
    let orders = create_resource(|| (), |_| async move { crate::api::store::list_orders().await });
    view! {
        <div class="sv-card">
            <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"Recent Orders"</h2>
            <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:140px;"></div> }>
                {move || orders.get().map(|res| match res {
                    Ok(rows) if rows.is_empty() => view! {
                        <div style="text-align:center;color:#A0A0B0;padding:24px;">"No orders yet."</div>
                    }.into_view(),
                    Ok(rows) => view! {
                        <table class="sv-table">
                            <thead><tr><th>"Order"</th><th>"Subtotal"</th><th>"Discount"</th><th>"Total"</th><th>"Created"</th></tr></thead>
                            <tbody>
                                {rows.into_iter().map(|o| view! {
                                    <tr>
                                        <td style="font-family:monospace;font-size:11px;">{o.id}</td>
                                        <td>{format!("${:.2}", o.subtotal)}</td>
                                        <td style="color:#10B981;">{format!("-${:.2}", o.discount_applied)}</td>
                                        <td><strong>{format!("${:.2}", o.total)}</strong></td>
                                        <td style="color:#A0A0B0;font-size:11px;">{o.created_at}</td>
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

fn tab_btn(
    label: &'static str,
    this_tab: StoreTab,
    current: ReadSignal<StoreTab>,
    setter: WriteSignal<StoreTab>,
) -> impl IntoView {
    view! {
        <button
            on:click=move |_| setter.set(this_tab)
            style=move || {
                let active = current.get() == this_tab;
                format!(
                    "padding:12px 20px;background:transparent;border:none;cursor:pointer;font-size:13px;font-weight:600;color:{};border-bottom:2px solid {};margin-bottom:-1px;",
                    if active { "#F5C518" } else { "#A0A0B0" },
                    if active { "#F5C518" } else { "transparent" }
                )
            }
        >
            {label}
        </button>
    }
}
