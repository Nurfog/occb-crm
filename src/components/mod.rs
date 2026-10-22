//! Componentes compartidos: barra de navegación y marco de página.

pub mod consent;
pub mod tags;

use dioxus::prelude::*;

use crate::{
    api,
    auth::use_auth,
    privacy::{save_consent, use_consent, ConsentState},
    route::Route,
};

use self::consent::ConsentBanner;

/// Barra superior propia: marca + lo esencial para visitas
/// (Inicio, Ayuda, Entrar, Crear cuenta) o el menú de la app con sesión.
#[component]
pub fn NavBar() -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let mut busy = use_signal(|| false);

    let session = auth.read().session.clone();

    rsx! {
        header { class: "topbar",
            div { class: "topbar-inner",
                Link { to: Route::Home {}, class: "brand", "OCCB · CRM" }
                if session.is_none() {
                    nav { class: "nav-links",
                        Link { to: Route::Home {}, class: "nav-link", "Inicio" }
                        Link { to: Route::Help {}, class: "nav-link", "Ayuda" }
                    }
                    div { class: "nav-user",
                        Link { to: Route::Login {}, class: "nav-link", "Entrar" }
                        Link {
                            to: Route::Register {},
                            class: "btn btn-primary nav-cta",
                            "Crear cuenta"
                        }
                    }
                } else {
                    nav { class: "nav-links",
                        Link { to: Route::Dashboard {}, class: "nav-link", "Panel" }
                        Link { to: Route::Contacts {}, class: "nav-link", "Contactos" }
                        Link { to: Route::Companies {}, class: "nav-link", "Empresas" }
                        Link { to: Route::Leads {}, class: "nav-link", "Prospectos" }
                        Link { to: Route::Deals {}, class: "nav-link", "Negocios" }
                        Link { to: Route::Activities {}, class: "nav-link", "Actividades" }
                        Link { to: Route::Documents {}, class: "nav-link", "Documentos" }
                        Link { to: Route::Reports {}, class: "nav-link", "Informes" }
                        Link { to: Route::Calendar {}, class: "nav-link", "Calendario" }
                        Link { to: Route::Email {}, class: "nav-link", "Email" }
                        Link { to: Route::Whatsapp {}, class: "nav-link", "WhatsApp" }
                        Link { to: Route::Search {}, class: "nav-link", "Buscar" }
                        Link {
                            to: Route::Notifications {},
                            class: "nav-link",
                            "Notificaciones"
                            UnreadBadge {}
                        }
                        Link { to: Route::Audit {}, class: "nav-link", "Auditoría" }
                        Link { to: Route::Admin {}, class: "nav-link", "Admin" }
                        Link { to: Route::Settings {}, class: "nav-link", "Ajustes" }
                        Link { to: Route::Users {}, class: "nav-link", "Usuarios" }
                        Link { to: Route::Help {}, class: "nav-link", "Ayuda" }
                    }
                    div { class: "nav-user",
                        match session {
                            Some(s) => {
                                let email = s.user.email.clone();
                                let access = s.access_token.clone();
                                let refresh = s.refresh_token.clone();
                                rsx! {
                                    span { class: "muted nav-email", "{email}" }
                                    button {
                                        class: "btn btn-small",
                                        disabled: *busy.read(),
                                        onclick: move |_| {
                                            let access_v = access.clone();
                                            let refresh_v = refresh.clone();
                                            busy.set(true);
                                            spawn(async move {
                                                // Mejor esfuerzo: aunque el backend falle,
                                                // la sesión local se cierra igual.
                                                let _ = api::logout(&access_v, &refresh_v).await;
                                                auth.write().session = None;
                                                busy.set(false);
                                                nav.push(Route::Home {});
                                            });
                                        },
                                        "Salir"
                                    }
                                }
                            }
                            None => rsx! {},
                        }
                    }
                }
            }
        }
    }
}

/// Insignia con el conteo de no leídas. Vacía si no hay sesión o es cero.
#[component]
fn UnreadBadge() -> Element {
    let auth = use_auth();
    let token = auth.read().session.clone().map(|s| s.access_token);
    let count = use_resource(move || {
        let t = token.clone();
        async move {
            match t {
                Some(tok) => api::unread_count(&tok).await.map(|c| c.count).unwrap_or(0),
                None => 0,
            }
        }
    });
    let n: i64 = count.read().clone().unwrap_or(0);
    if n > 0 {
        rsx! {
            span { class: "badge", "{n}" }
        }
    } else {
        rsx! {
            span {}
        }
    }
}

/// Marco de página: navegación + título + contenido + pie con privacidad.
#[component]
pub fn Layout(title: String, children: Element) -> Element {
    let consent = use_consent();
    let mut consent_w = consent;
    let decided = consent.read().decided_at.is_some();

    rsx! {
        div { class: "shell",
            NavBar {}
            main { class: "container",
                if !title.is_empty() {
                    h1 { class: "page-title", "{title}" }
                }
                {children}
            }
            footer { class: "footer footer-dark",
                div { class: "footer-grid",
                    div {
                        h4 { class: "footer-title", "Módulos" }
                        Link { to: Route::Leads {}, class: "footer-link", "Prospectos" }
                        Link { to: Route::Contacts {}, class: "footer-link", "Contactos" }
                        Link { to: Route::Companies {}, class: "footer-link", "Empresas" }
                        Link { to: Route::Deals {}, class: "footer-link", "Negocios" }
                        Link { to: Route::Activities {}, class: "footer-link", "Actividades" }
                        Link { to: Route::Documents {}, class: "footer-link", "Documentos" }
                    }
                    div {
                        h4 { class: "footer-title", "Producto" }
                        Link { to: Route::Dashboard {}, class: "footer-link", "Panel" }
                        Link { to: Route::Reports {}, class: "footer-link", "Informes" }
                        Link { to: Route::Calendar {}, class: "footer-link", "Calendario" }
                        Link { to: Route::Email {}, class: "footer-link", "Email" }
                        Link { to: Route::Whatsapp {}, class: "footer-link", "WhatsApp" }
                        Link { to: Route::Help {}, class: "footer-link", "Ayuda" }
                    }
                    div {
                        h4 { class: "footer-title", "Legal y privacidad" }
                        Link { to: Route::Privacy {}, class: "footer-link", "Política de Privacidad" }
                        Link { to: Route::Rights {}, class: "footer-link", "Tus derechos (ARCOP+B)" }
                        Link { to: Route::Status {}, class: "footer-link", "Estado del sistema" }
                        button {
                            class: "link-btn footer-link",
                            title: "Volver a elegir qué finalidades opcionales aceptas",
                            onclick: move |_| {
                                // Revocar: se borra la decisión y reaparece el banner.
                                let undecided = ConsentState::default();
                                save_consent(&undecided);
                                consent_w.set(undecided);
                            },
                            "Preferencias de privacidad"
                        }
                    }
                }
                div { class: "footer-bottom",
                    span { "© 2026 OCCB CRM" }
                    span { class: "muted", "Frontend Dioxus · API occb · Ley 21.719" }
                }
            }
            if !decided {
                ConsentBanner {}
            }
        }
    }
}
