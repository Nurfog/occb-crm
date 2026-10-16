//! OCCB CRM — frontend Dioxus (solo frontend).
//!
//! El backend vive en el proyecto `occb` (Axum + Postgres). Este crate solo
//! renderiza UI y consume su API REST (`/api/v1/...`). Ver README.md.

mod api;
mod auth;
mod components;
mod config;
mod pages;
mod privacy;
mod route;

use dioxus::prelude::*;

static MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    auth::provide_auth();
    privacy::provide_consent();
    rsx! {
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<route::Route> {}
    }
}
