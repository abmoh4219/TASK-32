//! Inline-SVG chart components — no JS interop, just Leptos rendering SVG
//! elements that render with the golden-gradient theme.

use leptos::*;

/// Simple line chart. `points` are `(label, value)` pairs.
#[component]
pub fn LineChart(
    #[prop(into)] title: String,
    points: Vec<(String, f64)>,
    #[prop(default = 320.0)] width: f64,
    #[prop(default = 180.0)] height: f64,
) -> impl IntoView {
    let pad = 30.0_f64;
    let max = points.iter().map(|(_, v)| *v).fold(0.0_f64, f64::max).max(1.0);
    let n = points.len().max(1);
    let step = if n > 1 {
        (width - pad * 2.0) / (n - 1) as f64
    } else {
        0.0
    };
    let path = if points.is_empty() {
        String::new()
    } else {
        let mut s = String::new();
        for (i, (_, v)) in points.iter().enumerate() {
            let x = pad + i as f64 * step;
            let y = height - pad - (v / max) * (height - pad * 2.0);
            if i == 0 {
                s.push_str(&format!("M{:.2},{:.2}", x, y));
            } else {
                s.push_str(&format!(" L{:.2},{:.2}", x, y));
            }
        }
        s
    };
    let labels = points.clone();

    view! {
        <div class="sv-card">
            <div style="font-size:12px;color:#A0A0B0;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:8px;">
                {title}
            </div>
            <svg width=width height=height style="display:block;">
                <rect x="0" y="0" width=width height=height fill="rgba(255,255,255,0.02)" rx="6"/>
                <path d=path stroke="#F5C518" stroke-width="2" fill="none"/>
                {labels.into_iter().enumerate().map(|(i, (label, v))| {
                    let x = pad + i as f64 * step;
                    let y = height - pad - (v / max) * (height - pad * 2.0);
                    view! {
                        <g>
                            <circle cx=x cy=y r="3" fill="#F5C518"/>
                            <text x=x y=height - 8.0 fill="#A0A0B0" font-size="9" text-anchor="middle">{label}</text>
                        </g>
                    }
                }).collect_view()}
            </svg>
        </div>
    }
}

/// Simple bar chart with optional cap line drawn at `cap`.
#[component]
pub fn BarChart(
    #[prop(into)] title: String,
    bars: Vec<(String, f64)>,
    #[prop(optional)] cap: Option<f64>,
    #[prop(default = 320.0)] width: f64,
    #[prop(default = 180.0)] height: f64,
) -> impl IntoView {
    let pad = 30.0_f64;
    let max = bars
        .iter()
        .map(|(_, v)| *v)
        .chain(cap.into_iter())
        .fold(0.0_f64, f64::max)
        .max(1.0);
    let n = bars.len().max(1);
    let bar_width = (width - pad * 2.0) / (n as f64 * 1.5).max(1.0);
    let bars_clone = bars.clone();

    view! {
        <div class="sv-card">
            <div style="font-size:12px;color:#A0A0B0;text-transform:uppercase;letter-spacing:0.05em;margin-bottom:8px;">
                {title}
            </div>
            <svg width=width height=height style="display:block;">
                <rect x="0" y="0" width=width height=height fill="rgba(255,255,255,0.02)" rx="6"/>
                {bars_clone.into_iter().enumerate().map(|(i, (label, v))| {
                    let x = pad + i as f64 * 1.5 * bar_width;
                    let bar_h = (v / max) * (height - pad * 2.0);
                    let y = height - pad - bar_h;
                    view! {
                        <g>
                            <rect x=x y=y width=bar_width height=bar_h fill="#F5C518" rx="2"/>
                            <text x=x + bar_width / 2.0 y=height - 8.0 fill="#A0A0B0" font-size="9" text-anchor="middle">{label}</text>
                        </g>
                    }
                }).collect_view()}
                {cap.map(|c| {
                    let cy = height - pad - (c / max) * (height - pad * 2.0);
                    view! {
                        <line x1=pad x2=width - pad y1=cy y2=cy stroke="#EF4444" stroke-width="2" stroke-dasharray="4,3"/>
                    }
                })}
            </svg>
        </div>
    }
}

/// Histogram — same shape as the bar chart but renders a frequency distribution.
#[component]
pub fn Histogram(
    #[prop(into)] title: String,
    buckets: Vec<(String, i64)>,
    #[prop(default = 320.0)] width: f64,
    #[prop(default = 180.0)] height: f64,
) -> impl IntoView {
    let bars: Vec<(String, f64)> = buckets
        .into_iter()
        .map(|(label, count)| (label, count as f64))
        .collect();
    view! { <BarChart title=title bars=bars width=width height=height/> }
}
