//! Store API client serialization tests.

use frontend::api::store::{
    CartItem, CheckoutRequest, CheckoutResponse, CreatePromotionInput, LineItemResult,
};

#[test]
fn test_checkout_request_serializes_cart_items() {
    let req = CheckoutRequest {
        items: vec![
            CartItem {
                product_id: "p1".into(),
                product_name: "Book".into(),
                quantity: 2,
                unit_price: 39.99,
            },
            CartItem {
                product_id: "p2".into(),
                product_name: "Kit".into(),
                quantity: 1,
                unit_price: 59.0,
            },
        ],
    };
    let v = serde_json::to_value(&req).unwrap();
    let arr = v["items"].as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["quantity"], 2);
    assert_eq!(arr[1]["unit_price"], 59.0);
}

#[test]
fn test_checkout_response_deserializes_line_items() {
    let body = r#"{
        "order": {
            "id":"o-1","user_id":"u-store","status":"completed",
            "subtotal":100.0,"discount_applied":10.0,"total":90.0,
            "created_at":"2026-04-01T00:00:00Z"
        },
        "result": {
            "line_items":[{
                "item":{"product_id":"p1","product_name":"Book","quantity":2,"unit_price":50.0},
                "line_subtotal":100.0,
                "discount_amount":10.0,
                "line_total":90.0,
                "promotion_applied":"Spring 10% Off"
            }],
            "subtotal":100.0,
            "total_discount":10.0,
            "total":90.0,
            "best_promotion":null
        }
    }"#;
    let parsed: CheckoutResponse = serde_json::from_str(body).unwrap();
    assert_eq!(parsed.order.total, 90.0);
    assert_eq!(parsed.result.line_items.len(), 1);
    assert_eq!(
        parsed.result.line_items[0].promotion_applied.as_deref(),
        Some("Spring 10% Off")
    );
}

#[test]
fn test_line_item_result_round_trip() {
    let line = LineItemResult {
        item: CartItem {
            product_id: "p".into(),
            product_name: "X".into(),
            quantity: 1,
            unit_price: 10.0,
        },
        line_subtotal: 10.0,
        discount_amount: 1.0,
        line_total: 9.0,
        promotion_applied: None,
    };
    let json = serde_json::to_string(&line).unwrap();
    assert!(json.contains("line_subtotal"));
    assert!(json.contains("discount_amount"));
    let back: LineItemResult = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line_total, 9.0);
}

#[test]
fn test_create_promotion_input_serializes_all_fields() {
    let input = CreatePromotionInput {
        name: "Spring".into(),
        description: "Site-wide".into(),
        discount_value: 10.0,
        discount_type: "percent".into(),
        effective_from: "2026-04-01T00:00:00Z".into(),
        effective_until: "2026-04-30T23:59:59Z".into(),
        mutual_exclusion_group: Some("site-wide".into()),
        priority: 5,
    };
    let v = serde_json::to_value(&input).unwrap();
    for k in [
        "name",
        "description",
        "discount_value",
        "discount_type",
        "effective_from",
        "effective_until",
        "mutual_exclusion_group",
        "priority",
    ] {
        assert!(v.get(k).is_some(), "missing key {}", k);
    }
}
