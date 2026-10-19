//! Mapa de rutas del frontend (Dioxus Router).

use dioxus::prelude::*;

use crate::pages::{
    auth_pages::{Login, Register},
    contacts::{ContactDetail, Contacts},
    dashboard::Dashboard,
    deals::{DealDetail, Deals},
    home::Home,
    not_found::NotFound,
    privacy::Privacy,
    rights::Rights,
    status::Status,
    users::{UserDetail, Users},
};

/// Rutas de la SPA. Solo frontend: cada ruta consume el API del backend occb.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/dashboard")]
    Dashboard {},
    #[route("/contactos")]
    Contacts {},
    #[route("/contactos/:id")]
    ContactDetail { id: String },
    #[route("/negocios")]
    Deals {},
    #[route("/negocios/:id")]
    DealDetail { id: String },
    #[route("/users")]
    Users {},
    #[route("/users/:id")]
    UserDetail { id: String },
    #[route("/status")]
    Status {},
    #[route("/privacidad")]
    Privacy {},
    #[route("/derechos")]
    Rights {},
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
