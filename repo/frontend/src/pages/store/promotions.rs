//! Promotions tab — list + create-promotion form (MM/DD/YYYY 12-hour display).

use leptos::*;
use wasm_bindgen_futures::spawn_local;

use crate::api::store::{self as store_api, CreatePromotionInput};
use crate::logic::promotion::{format_discount, iso_to_mmddyyyy, mmddyyyy_12h_to_iso};

#[component]
pub fn PromotionsTab() -> impl IntoView {
    let promos = create_resource(|| (), |_| async move { store_api::list_promotions().await });

    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (discount_value, set_discount_value) = create_signal(10.0_f64);
    let (discount_type, set_discount_type) = create_signal("percent".to_string());
    let (from_date, set_from_date) = create_signal(String::new());
    let (from_time, set_from_time) = create_signal(String::new());
    let (from_meridiem, set_from_meridiem) = create_signal("AM".to_string());
    let (until_date, set_until_date) = create_signal(String::new());
    let (until_time, set_until_time) = create_signal(String::new());
    let (until_meridiem, set_until_meridiem) = create_signal("PM".to_string());
    let (group, set_group) = create_signal(String::new());
    let (priority, set_priority) = create_signal(0i64);
    let (status, set_status) = create_signal::<Option<String>>(None);

    let create = move |_| {
        if name.get().trim().is_empty() {
            set_status.set(Some("Name is required".into()));
            return;
        }
        let from = match mmddyyyy_12h_to_iso(
            &from_date.get(),
            &from_time.get(),
            &from_meridiem.get(),
        ) {
            Some(s) => s,
            None => {
                set_status.set(Some(
                    "Invalid \"Effective from\" — use MM/DD/YYYY and hh:mm AM/PM".into(),
                ));
                return;
            }
        };
        let until = match mmddyyyy_12h_to_iso(
            &until_date.get(),
            &until_time.get(),
            &until_meridiem.get(),
        ) {
            Some(s) => s,
            None => {
                set_status.set(Some(
                    "Invalid \"Effective until\" — use MM/DD/YYYY and hh:mm AM/PM".into(),
                ));
                return;
            }
        };
        let group_val = group.get();
        let payload = CreatePromotionInput {
            name: name.get(),
            description: description.get(),
            discount_value: discount_value.get(),
            discount_type: discount_type.get(),
            effective_from: from,
            effective_until: until,
            mutual_exclusion_group: if group_val.is_empty() { None } else { Some(group_val) },
            priority: priority.get(),
        };
        spawn_local(async move {
            match store_api::create_promotion(payload).await {
                Ok(p) => {
                    set_status.set(Some(format!("Created \"{}\"", p.name)));
                    set_name.set(String::new());
                    promos.refetch();
                }
                Err(e) => set_status.set(Some(format!("Error: {}", e.message))),
            }
        });
    };

    view! {
        <div style="display:grid;grid-template-columns:1.5fr 1fr;gap:24px;">
            <div class="sv-card">
                <h2 style="margin:0 0 16px;font-size:16px;color:#F5C518;">"Active Promotions"</h2>
                <Suspense fallback=|| view! { <div class="sv-skeleton" style="height:160px;"></div> }>
                    {move || promos.get().map(|res| match res {
                        Ok(rows) if rows.is_empty() => view! {
                            <div style="text-align:center;color:#A0A0B0;padding:24px;">"No promotions configured."</div>
                        }.into_view(),
                        Ok(rows) => view! {
                            <table class="sv-table">
                                <thead>
                                    <tr>
                                        <th>"Name"</th>
                                        <th>"Discount"</th>
                                        <th>"Window"</th>
                                        <th>"Group"</th>
                                        <th>"Priority"</th>
                                        <th>"Status"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|p| {
                                        let active_badge = if p.is_active == 1 { "sv-badge sv-badge-success" } else { "sv-badge sv-badge-danger" };
                                        let active_label = if p.is_active == 1 { "active" } else { "inactive" };
                                        view! {
                                            <tr>
                                                <td>
                                                    <div style="font-weight:600;">{p.name}</div>
                                                    <div style="font-size:11px;color:#A0A0B0;">{p.description}</div>
                                                </td>
                                                <td><span class="sv-badge sv-badge-warning">{format_discount(&p.discount_type, p.discount_value)}</span></td>
                                                <td style="font-size:11px;">
                                                    <div>{iso_to_mmddyyyy(&p.effective_from)}</div>
                                                    <div style="color:#A0A0B0;">{format!("→ {}", iso_to_mmddyyyy(&p.effective_until))}</div>
                                                </td>
                                                <td style="font-family:monospace;font-size:11px;">{p.mutual_exclusion_group.unwrap_or_else(|| "—".into())}</td>
                                                <td>{p.priority}</td>
                                                <td><span class=active_badge>{active_label}</span></td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_view(),
                        Err(e) => view! { <div class="sv-error">{format!("Failed: {}", e.message)}</div> }.into_view(),
                    })}
                </Suspense>
            </div>

            <div class="sv-card">
                <h3 style="margin:0 0 12px;font-size:14px;color:#F5C518;">"New Promotion"</h3>
                <label class="sv-label">"Name"</label>
                <input class="sv-input" prop:value=move || name.get()
                    on:input=move |ev| set_name.set(event_target_value(&ev))/>
                <label class="sv-label" style="margin-top:10px;">"Description"</label>
                <input class="sv-input" prop:value=move || description.get()
                    on:input=move |ev| set_description.set(event_target_value(&ev))/>

                <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;margin-top:10px;">
                    <div>
                        <label class="sv-label">"Type"</label>
                        <select
                            class="sv-input"
                            on:change=move |ev| set_discount_type.set(event_target_value(&ev))
                        >
                            <option value="percent">"% Percent"</option>
                            <option value="fixed">"$ Fixed"</option>
                        </select>
                    </div>
                    <div>
                        <label class="sv-label">"Value"</label>
                        <input class="sv-input" type="number" step="0.01"
                            prop:value=move || discount_value.get().to_string()
                            on:input=move |ev| set_discount_value.set(event_target_value(&ev).parse().unwrap_or(0.0))/>
                    </div>
                </div>

                <label class="sv-label" style="margin-top:10px;">"Effective from (MM/DD/YYYY, hh:mm AM/PM)"</label>
                <div style="display:grid;grid-template-columns:1.3fr 1fr 0.7fr;gap:6px;">
                    <input class="sv-input" placeholder="MM/DD/YYYY"
                        prop:value=move || from_date.get()
                        on:input=move |ev| set_from_date.set(event_target_value(&ev))/>
                    <input class="sv-input" placeholder="hh:mm"
                        prop:value=move || from_time.get()
                        on:input=move |ev| set_from_time.set(event_target_value(&ev))/>
                    <select class="sv-input"
                        on:change=move |ev| set_from_meridiem.set(event_target_value(&ev))>
                        <option value="AM">"AM"</option>
                        <option value="PM">"PM"</option>
                    </select>
                </div>
                <label class="sv-label" style="margin-top:10px;">"Effective until (MM/DD/YYYY, hh:mm AM/PM)"</label>
                <div style="display:grid;grid-template-columns:1.3fr 1fr 0.7fr;gap:6px;">
                    <input class="sv-input" placeholder="MM/DD/YYYY"
                        prop:value=move || until_date.get()
                        on:input=move |ev| set_until_date.set(event_target_value(&ev))/>
                    <input class="sv-input" placeholder="hh:mm"
                        prop:value=move || until_time.get()
                        on:input=move |ev| set_until_time.set(event_target_value(&ev))/>
                    <select class="sv-input"
                        on:change=move |ev| set_until_meridiem.set(event_target_value(&ev))>
                        <option value="AM">"AM"</option>
                        <option value="PM" selected=true>"PM"</option>
                    </select>
                </div>

                <div style="display:grid;grid-template-columns:1.5fr 1fr;gap:8px;margin-top:10px;">
                    <div>
                        <label class="sv-label">"Mutual exclusion group"</label>
                        <input class="sv-input" prop:value=move || group.get()
                            on:input=move |ev| set_group.set(event_target_value(&ev))/>
                    </div>
                    <div>
                        <label class="sv-label">"Priority"</label>
                        <input class="sv-input" type="number"
                            prop:value=move || priority.get().to_string()
                            on:input=move |ev| set_priority.set(event_target_value(&ev).parse().unwrap_or(0))/>
                    </div>
                </div>

                <button class="sv-btn-primary" style="margin-top:18px;width:100%;" on:click=create>
                    "Create Promotion"
                </button>

                {move || status.get().map(|s| view! {
                    <div style="margin-top:12px;font-size:11px;color:#A0A0B0;">{s}</div>
                })}
            </div>
        </div>
    }
}
