//! Backend unit tests for the promotion engine.

use backend::models::store::Promotion;
use backend::services::store_service::{
    apply_best_promotion, in_window, resolve_exclusion_groups, total_cart_discount, CartItem,
};
use chrono::{Duration, Utc};

fn promo(
    id: &str,
    name: &str,
    discount_value: f64,
    discount_type: &str,
    group: Option<&str>,
    priority: i64,
) -> Promotion {
    let now = Utc::now();
    Promotion {
        id: id.into(),
        name: name.into(),
        description: String::new(),
        discount_value,
        discount_type: discount_type.into(),
        effective_from: (now - Duration::days(1)).to_rfc3339(),
        effective_until: (now + Duration::days(7)).to_rfc3339(),
        mutual_exclusion_group: group.map(|s| s.to_string()),
        priority,
        is_active: 1,
        created_by: "test".into(),
        created_at: now.to_rfc3339(),
    }
}

fn cart() -> Vec<CartItem> {
    vec![
        CartItem {
            product_id: "p1".into(),
            product_name: "Book".into(),
            quantity: 2,
            unit_price: 50.0,
        },
        CartItem {
            product_id: "p2".into(),
            product_name: "Kit".into(),
            quantity: 1,
            unit_price: 30.0,
        },
    ]
}

#[test]
fn test_best_offer_selects_highest_priority_in_group() {
    // Two promos in the same group with different priorities — only the
    // higher-priority promo survives mutual-exclusion resolution.
    let p_low = promo("p-low", "10% off", 10.0, "percent", Some("site"), 1);
    let p_high = promo("p-high", "15% off", 15.0, "percent", Some("site"), 5);
    let resolved = resolve_exclusion_groups(&[&p_low, &p_high]);
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].id, "p-high");
}

#[test]
fn test_mutual_exclusion_one_per_group() {
    let p_a1 = promo("a1", "5% A", 5.0, "percent", Some("group-a"), 1);
    let p_a2 = promo("a2", "10% A", 10.0, "percent", Some("group-a"), 9);
    let p_b1 = promo("b1", "$5 B", 5.0, "fixed", Some("group-b"), 2);
    let resolved = resolve_exclusion_groups(&[&p_a1, &p_a2, &p_b1]);
    assert_eq!(resolved.len(), 2);
    let ids: Vec<&str> = resolved.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"a2"));
    assert!(ids.contains(&"b1"));
    assert!(!ids.contains(&"a1"));
}

#[test]
fn test_two_exclusion_groups_each_gets_best() {
    // Across the surviving set, apply_best_promotion picks the single best.
    let p_a = promo("a", "20% A", 20.0, "percent", Some("group-a"), 9);
    let p_b = promo("b", "$5 B", 5.0, "fixed", Some("group-b"), 9);
    let result = apply_best_promotion(&cart(), &[p_a.clone(), p_b]);
    // Cart subtotal = 130.0; 20% = 26.0 > $5 fixed
    assert_eq!(result.best_promotion.unwrap().id, "a");
    assert!((result.total_discount - 26.0).abs() < 1e-6);
}

#[test]
fn test_expired_promotion_not_applied() {
    let now = Utc::now();
    let mut p = promo("expired", "50% off", 50.0, "percent", None, 9);
    p.effective_from = (now - Duration::days(30)).to_rfc3339();
    p.effective_until = (now - Duration::days(1)).to_rfc3339();
    assert!(!in_window(&p, Utc::now()));
    let result = apply_best_promotion(&cart(), &[p]);
    assert!(result.best_promotion.is_none());
}

#[test]
fn test_future_promotion_not_applied() {
    let now = Utc::now();
    let mut p = promo("future", "50% off", 50.0, "percent", None, 9);
    p.effective_from = (now + Duration::days(1)).to_rfc3339();
    p.effective_until = (now + Duration::days(10)).to_rfc3339();
    assert!(!in_window(&p, Utc::now()));
    let result = apply_best_promotion(&cart(), &[p]);
    assert_eq!(result.total_discount, 0.0);
}

#[test]
fn test_promotion_at_boundary_applied() {
    // A promo whose effective_from is exactly now should be in-window.
    let now = Utc::now();
    let mut p = promo("boundary", "5% off", 5.0, "percent", None, 1);
    p.effective_from = (now - Duration::seconds(1)).to_rfc3339();
    p.effective_until = (now + Duration::seconds(1)).to_rfc3339();
    assert!(in_window(&p, Utc::now()));
}

#[test]
fn test_line_item_trace_contains_promotion_name() {
    let p = promo("named", "Spring Sale", 10.0, "percent", None, 1);
    let result = apply_best_promotion(&cart(), &[p]);
    assert!(result.line_items.iter().all(|l| {
        l.promotion_applied
            .as_deref()
            .map(|n| n == "Spring Sale")
            .unwrap_or(false)
    }));
}

#[test]
fn test_total_cart_discount_percent() {
    let p = promo("p10", "10%", 10.0, "percent", None, 0);
    let total = total_cart_discount(&cart(), &p); // 130 * 0.10 = 13
    assert!((total - 13.0).abs() < 1e-9);
}

#[test]
fn test_total_cart_discount_fixed() {
    let p = promo("p5", "$5 off", 5.0, "fixed", None, 0);
    let total = total_cart_discount(&cart(), &p);
    assert_eq!(total, 5.0);
}

#[test]
fn test_fixed_promo_capped_at_subtotal() {
    let small_cart = vec![CartItem {
        product_id: "p".into(),
        product_name: "Snack".into(),
        quantity: 1,
        unit_price: 3.0,
    }];
    let p = promo("big", "$50 off", 50.0, "fixed", None, 0);
    let total = total_cart_discount(&small_cart, &p);
    assert_eq!(total, 3.0, "fixed discount should never exceed the subtotal");
}
