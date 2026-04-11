//! Checkout tab — pick products, build a cart, run "Apply Best Offer" against
//! the backend promotion engine, see per-line discount trace.

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::store::{self as store_api, CartItem, CheckoutResult, Product};

#[component]
pub fn CheckoutTab() -> impl IntoView {
    let products = create_resource(|| (), |_| async move { store_api::list_products().await });
    let (cart, set_cart) = create_signal::<Vec<CartItem>>(Vec::new());
    let (preview, set_preview) = create_signal::<Option<CheckoutResult>>(None);
    let (status, set_status) = create_signal::<Option<String>>(None);

    let add_to_cart = move |p: Product| {
        set_cart.update(|c| {
            if let Some(existing) = c.iter_mut().find(|i| i.product_id == p.id) {
                existing.quantity += 1;
            } else {
                c.push(CartItem {
                    product_id: p.id.clone(),
                    product_name: p.name.clone(),
                    quantity: 1,
                    unit_price: p.price,
                });
            }
        });
    };

    let apply_offer = move |_| {
        let items = cart.get();
        if items.is_empty() {
            set_status.set(Some("Cart is empty".into()));
            return;
        }
        spawn_local(async move {
            match store_api::preview_checkout(items).await {
                Ok(r) => {
                    set_preview.set(Some(r));
                    set_status.set(Some("Best offer applied".into()));
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    let complete_checkout = move |_| {
        let items = cart.get();
        if items.is_empty() {
            return;
        }
        spawn_local(async move {
            match store_api::checkout(items).await {
                Ok(resp) => {
                    set_status.set(Some(format!("Order {} created (${:.2})", resp.order.id, resp.order.total)));
                    set_cart.set(Vec::new());
                    set_preview.set(None);
                }
                Err(e) => set_status.set(Some(format!("Checkout failed: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1fr 1.2fr;gap:24px;">
            <div class="sv-card">
                <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"Catalog"</h2>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:200px;"></div> }>
                    {move || products.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div style="text-align:center;color:#A0A0B0;padding:24px;">"No products."</div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <div style="display:flex;flex-direction:column;gap:10px;">
                                {rows.into_iter().map(|p| {
                                    let p_clone = p.clone();
                                    view! {
                                        <div style="display:flex;align-items:center;justify-content:space-between;padding:12px;background:rgba(255,255,255,0.02);border-radius:8px;border:1px solid rgba(245,197,24,0.10);">
                                            <div>
                                                <div style="font-weight:600;font-size:13px;">{p.name.clone()}</div>
                                                <div style="font-size:11px;color:#A0A0B0;">{p.description.clone()}</div>
                                            </div>
                                            <div style="display:flex;align-items:center;gap:12px;">
                                                <span style="font-weight:600;color:#F5C518;">{format!("${:.2}", p.price)}</span>
                                                <button class="sv-btn-secondary" style="padding:6px 12px;font-size:11px;"
                                                    on:click=move |_| add_to_cart(p_clone.clone())>
                                                    "Add"
                                                </button>
                                            </div>
                                        </div>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{format!("Failed: {}", e.message)}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div>
                <div class="sv-card" style="margin-bottom:16px;">
                    <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"Cart"</h2>
                    {move || {
                        let items = cart.get();
                        if items.is_empty() {
                            view! { <div style="color:#A0A0B0;text-align:center;padding:16px;">"Empty cart"</div> }.into_view()
                        } else {
                            view! {
                                <table class="sv-table">
                                    <thead><tr><th>"Product"</th><th>"Qty"</th><th>"Price"</th><th>"Subtotal"</th></tr></thead>
                                    <tbody>
                                        {items.iter().map(|i| view! {
                                            <tr>
                                                <td>{i.product_name.clone()}</td>
                                                <td>{i.quantity}</td>
                                                <td>{format!("${:.2}", i.unit_price)}</td>
                                                <td>{format!("${:.2}", i.unit_price * i.quantity as f64)}</td>
                                            </tr>
                                        }).collect_view()}
                                    </tbody>
                                </table>
                            }.into_view()
                        }
                    }}
                    <div style="display:flex;gap:8px;margin-top:14px;">
                        <button class="sv-btn-secondary" on:click=apply_offer>"Apply Best Offer"</button>
                        <button class="sv-btn-primary" on:click=complete_checkout>"Complete Checkout"</button>
                    </div>
                    {move || status.get().map(|s| view! {
                        <div style="margin-top:10px;font-size:11px;color:#A0A0B0;">{s}</div>
                    })}
                </div>

                {move || preview.get().map(|r| {
                    let promo_name = r.best_promotion.as_ref().map(|p| p.name.clone()).unwrap_or_else(|| "—".into());
                    view! {
                        <div class="sv-card">
                            <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"Discount Breakdown"</h3>
                            <div style="font-size:12px;color:#A0A0B0;margin-bottom:10px;">
                                {format!("Best promotion: {}", promo_name)}
                            </div>
                            <table class="sv-table">
                                <thead><tr><th>"Line"</th><th>"Subtotal"</th><th>"Discount"</th><th>"Total"</th><th>"Promotion"</th></tr></thead>
                                <tbody>
                                    {r.line_items.iter().map(|l| view! {
                                        <tr>
                                            <td>{l.item.product_name.clone()}</td>
                                            <td>{format!("${:.2}", l.line_subtotal)}</td>
                                            <td style="color:#10B981;">{format!("-${:.2}", l.discount_amount)}</td>
                                            <td><strong>{format!("${:.2}", l.line_total)}</strong></td>
                                            <td style="font-size:11px;color:#A0A0B0;">{l.promotion_applied.clone().unwrap_or_else(|| "—".into())}</td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                            </table>
                            <div style="display:flex;justify-content:space-between;margin-top:14px;padding-top:14px;border-top:1px solid rgba(245,197,24,0.20);font-size:14px;font-weight:600;">
                                <span>"Total"</span>
                                <span style="color:#F5C518;">{format!("${:.2} (saved ${:.2})", r.total, r.total_discount)}</span>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}
