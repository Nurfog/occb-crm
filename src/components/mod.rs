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
                        Link { to: Route::Search { q: String::new() }, class: "nav-link", "Buscar" }
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

/// Marco de página: shell público (landing) o shell de app con sidebar.
/// El pie con privacidad va siempre.
#[component]
pub fn Layout(title: String, children: Element) -> Element {
    let auth = use_auth();
    let logged = auth.read().session.is_some();

    if logged {
        rsx! {
            AppShell { title: title, children: children }
        }
    } else {
        rsx! {
            PublicShell { title: title, children: children }
        }
    }
}

/// Ruta canónica para marcar el enlace activo (el detalle activa su lista).
fn canonical(route: &Route) -> Route {
    match route {
        Route::ContactDetail { .. } => Route::Contacts {},
        Route::CompanyDetail { .. } => Route::Companies {},
        Route::LeadDetail { .. } => Route::Leads {},
        Route::DealDetail { .. } => Route::Deals {},
        Route::UserDetail { .. } => Route::Users {},
        _ => route.clone(),
    }
}

/// Shell de la app: sidebar + header + contenido, como el original.
#[component]
/// Cabecera de la app: menú móvil, buscador global, campana,
/// botón +Nuevo y avatar con menú (Mi cuenta, Salir). Como el original.
#[component]
fn AppHeader(title: String, on_menu: EventHandler<()>) -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let mut busy = use_signal(|| false);
    let mut user_open = use_signal(|| false);
    let mut new_open = use_signal(|| false);
    let mut q = use_signal(String::new);

    let session = auth.read().session.clone();
    let user_is_open = *user_open.read();
    let new_is_open = *new_open.read();
    let initials = session
        .as_ref()
        .map(|s| {
            s.user
                .name
                .split_whitespace()
                .filter_map(|w| w.chars().next())
                .take(2)
                .collect::<String>()
                .to_uppercase()
        })
        .unwrap_or_default();

    rsx! {
        header { class: "flex h-14 flex-shrink-0 items-center gap-2 border-b border-border bg-card px-4",
            button {
                class: "rounded-lg p-2 text-muted-foreground hover:bg-muted hover:text-foreground lg:hidden",
                onclick: move |_| on_menu.call(()),
                "☰"
            }
            span { class: "hidden truncate text-sm font-semibold md:block lg:hidden", "{title}" }
            form {
                class: "mx-auto w-full max-w-lg flex-1 px-2",
                onsubmit: move |e| {
                    e.prevent_default();
                    let query = q.read().trim().to_string();
                    nav.push(Route::Search { q: query });
                },
                input {
                    id: "global-search",
                    class: "w-full rounded-lg border border-border bg-background px-3 py-1.5 text-sm outline-none placeholder:text-muted-foreground focus:border-primary",
                    r#type: "search",
                    placeholder: "Buscar contactos, empresas, negocios…",
                    value: "{q}",
                    oninput: move |e| q.set(e.value()),
                }
            }
            div { class: "ml-auto flex items-center gap-1",
                Link {
                    to: Route::Notifications {},
                    class: "relative rounded-lg p-2 text-muted-foreground hover:bg-muted hover:text-foreground",
                    title: "Notificaciones",
                    svg {
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        class: "h-5 w-5",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M14.857 17.082a23.848 23.848 0 005.454-1.31A8.967 8.967 0 0118 9.75v-.7V9A6 6 0 006 9v.75a8.967 8.967 0 01-2.312 6.022c1.733.64 3.56 1.085 5.455 1.31m5.714 0a24.255 24.255 0 01-5.714 0m5.714 0a3 3 0 11-5.714 0",
                        }
                    }
                    UnreadBadge {}
                }
                div { class: "relative",
                    button {
                        class: "rounded-lg bg-primary px-3 py-1.5 text-sm font-semibold text-primary-foreground hover:opacity-90",
                        onclick: move |_| new_open.set(!new_is_open),
                        "+ Nuevo"
                    }
                    if *new_open.read() {
                        div { class: "absolute right-0 z-50 mt-1 w-44 rounded-lg border border-border bg-card p-1 shadow-lg",
                            Link {
                                to: Route::Contacts {},
                                class: "block rounded-md px-3 py-2 text-sm hover:bg-muted",
                                onclick: move |_| new_open.set(false),
                                "Contacto"
                            }
                            Link {
                                to: Route::Companies {},
                                class: "block rounded-md px-3 py-2 text-sm hover:bg-muted",
                                onclick: move |_| new_open.set(false),
                                "Empresa"
                            }
                            Link {
                                to: Route::Deals {},
                                class: "block rounded-md px-3 py-2 text-sm hover:bg-muted",
                                onclick: move |_| new_open.set(false),
                                "Negocio"
                            }
                        }
                    }
                }
                match session {
                    Some(s) => {
                        let email = s.user.email.clone();
                        let access = s.access_token.clone();
                        let refresh = s.refresh_token.clone();
                        rsx! {
                            div { class: "relative",
                                button {
                                    class: "flex h-8 w-8 items-center justify-center rounded-full bg-primary text-xs font-bold text-white",
                                    title: "{email}",
                                    onclick: move |_| user_open.set(!user_is_open),
                                    "{initials}"
                                }
                                if *user_open.read() {
                                    div { class: "absolute right-0 z-50 mt-1 w-52 rounded-lg border border-border bg-card p-1 shadow-lg",
                                        p { class: "truncate px-3 py-2 text-xs text-muted-foreground", "{email}" }
                                        Link {
                                            to: Route::Account {},
                                            class: "block rounded-md px-3 py-2 text-sm hover:bg-muted",
                                            onclick: move |_| user_open.set(false),
                                            "Mi cuenta"
                                        }
                                        button {
                                            class: "block w-full rounded-md px-3 py-2 text-left text-sm hover:bg-muted disabled:opacity-50",
                                            disabled: *busy.read(),
                                            onclick: move |_| {
                                                let access_v = access.clone();
                                                let refresh_v = refresh.clone();
                                                busy.set(true);
                                                spawn(async move {
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
                            }
                        }
                    }
                    None => rsx! {},
                }
            }
        }
    }
}
/// Shell de la app: sidebar + header + contenido, como el original.
#[component]
fn AppShell(title: String, children: Element) -> Element {
    let mut open = use_signal(|| false);
    let current = canonical(&use_route::<Route>());
    let link_cls = |to: &Route| {
        if current == *to {
            "flex items-center gap-2 rounded-lg bg-sidebar-active px-3 py-2 text-sm font-semibold text-foreground"
        } else {
            "flex items-center gap-2 rounded-lg px-3 py-2 text-sm text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        }
    };

    rsx! {
        div { class: "flex h-screen overflow-hidden bg-background text-foreground",
            if *open.read() {
                div {
                    class: "fixed inset-0 z-30 bg-black/50 lg:hidden",
                    onclick: move |_| open.set(false),
                }
            }
            aside { class: {
                format!(
                    "fixed inset-y-0 left-0 z-40 flex w-64 flex-col border-r border-border bg-sidebar transition-transform duration-200 lg:static {}",
                    if *open.read() { "translate-x-0" } else { "-translate-x-full lg:translate-x-0" }
                )
            },
                div { class: "flex h-14 flex-shrink-0 items-center gap-2.5 border-b border-border px-4",
                    div { class: "flex h-8 w-8 items-center justify-center rounded-[10px] bg-primary text-[15px] font-bold text-white",
                        "O"
                    }
                    div { class: "flex flex-col leading-none",
                        span { class: "text-[14.5px] font-semibold tracking-tight", "OCCB CRM" }
                        span { class: "mt-0.5 text-[11px] text-muted-foreground", "Workspace" }
                    }
                    button {
                        class: "ml-auto rounded-lg p-1.5 text-muted-foreground hover:bg-muted lg:hidden",
                        onclick: move |_| open.set(false),
                        "✕"
                    }
                }
                nav { class: "flex-1 overflow-y-auto p-3",
                    p { class: "px-3 pt-1 pb-1 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase",
                        "Principal"
                    }
                    Link { to: Route::Dashboard {}, class: link_cls(&Route::Dashboard {}), "Panel" }
                    p { class: "px-3 pt-4 pb-1 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase",
                        "CRM"
                    }
                    Link { to: Route::Contacts {}, class: link_cls(&Route::Contacts {}), "Contactos" }
                    Link { to: Route::Companies {}, class: link_cls(&Route::Companies {}), "Empresas" }
                    Link { to: Route::Deals {}, class: link_cls(&Route::Deals {}), "Negocios" }
                    Link { to: Route::Leads {}, class: link_cls(&Route::Leads {}), "Prospectos" }
                    Link { to: Route::Email {}, class: link_cls(&Route::Email {}), "Email" }
                    Link { to: Route::Calendar {}, class: link_cls(&Route::Calendar {}), "Calendario" }
                    Link { to: Route::Activities {}, class: link_cls(&Route::Activities {}), "Actividades" }
                    Link { to: Route::Reports {}, class: link_cls(&Route::Reports {}), "Informes" }
                    Link { to: Route::Documents {}, class: link_cls(&Route::Documents {}), "Documentos" }
                    Link { to: Route::Whatsapp {}, class: link_cls(&Route::Whatsapp {}), "WhatsApp" }
                    p { class: "px-3 pt-4 pb-1 text-[11px] font-semibold tracking-wider text-muted-foreground uppercase",
                        "Sistema"
                    }
                    Link { to: Route::Settings {}, class: link_cls(&Route::Settings {}), "Ajustes" }
                    Link { to: Route::Admin {}, class: link_cls(&Route::Admin {}), "Admin" }
                    Link { to: Route::Help {}, class: link_cls(&Route::Help {}), "Ayuda" }
                    Link { to: Route::Audit {}, class: link_cls(&Route::Audit {}), "Auditoría" }
                }
            }
            div { class: "flex min-w-0 flex-1 flex-col",
                AppHeader {
                    title: title.clone(),
                    on_menu: move |_| open.set(true),
                }
                main { class: "flex-1 overflow-y-auto",
                    div { class: "mx-auto w-full max-w-[1360px] p-4 sm:p-6 lg:p-8",
                        if !title.is_empty() {
                            h1 { class: "mb-4 text-2xl font-bold tracking-tight", "{title}" }
                        }
                        {children}
                    }
                }
            }
        }
    }
}

/// Shell público: topbar mínima + contenido + pie con privacidad.
#[component]
fn PublicShell(title: String, children: Element) -> Element {
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
