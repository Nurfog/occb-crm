//! Mapa de rutas del frontend (Dioxus Router).

use dioxus::prelude::*;

use crate::pages::{
    account::Account,
    activities::Activities,
    admin::Admin,
    audit::Audit,
    auth_pages::{Login, Register},
    calendar::Calendar,
    companies::{Companies, CompanyDetail},
    contacts::{ContactDetail, Contacts},
    dashboard::Dashboard,
    deals::{DealDetail, Deals},
    documents::Documents,
    email::Email,
    help::Help,
    home::Home,
    leads::{LeadDetail, Leads},
    notifications::Notifications,
    password::{ForgotPassword, ResetPassword},
    reports::Reports,
    search::Search,
    settings::Settings,
    whatsapp::{Whatsapp, WhatsappConfig},
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
    #[route("/empresas")]
    Companies {},
    #[route("/empresas/:id")]
    CompanyDetail { id: String },
    #[route("/prospectos")]
    Leads {},
    #[route("/prospectos/:id")]
    LeadDetail { id: String },
    #[route("/actividades")]
    Activities {},
    #[route("/documentos")]
    Documents {},
    #[route("/informes")]
    Reports {},
    #[route("/notificaciones")]
    Notifications {},
    #[route("/cuenta")]
    Account {},
    #[route("/buscar?:q")]
    Search { q: String },
    #[route("/auditoria")]
    Audit {},
    #[route("/admin")]
    Admin {},
    #[route("/email")]
    Email {},
    #[route("/calendario")]
    Calendar {},
    #[route("/whatsapp")]
    Whatsapp {},
    #[route("/whatsapp/config")]
    WhatsappConfig {},
    #[route("/ajustes")]
    Settings {},
    #[route("/ayuda")]
    Help {},
    #[route("/recuperar")]
    ForgotPassword {},
    #[route("/restablecer")]
    ResetPassword {},
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
