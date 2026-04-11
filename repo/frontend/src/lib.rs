//! ScholarVault frontend library crate.
//!
//! Hosts the Leptos `App` component, the API client modules, and pure Rust helper
//! logic (form validation, masking, filter state) so the integration tests under
//! `tests/` can exercise that logic without a WASM runtime.

pub mod app;
pub mod api;
pub mod logic;
pub mod components;
pub mod pages;

pub use app::App;

/// Mount the Leptos `App` to the document body. Called from the binary entrypoint.
pub fn run() {
    console_error_panic_hook::set_once();
    leptos::mount_to_body(|| leptos::view! { <App /> });
}
