//! Panel: perfil (`GET /api/v1/auth/me`) y gestión de la sesión.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::{use_auth, Session},
    components::Layout,
    route::Route,
};

#[component]
pub fn Dashboard() -> Element {
    let mut auth = use_auth();
    let nav = use_navigator();
    let mut action_error = use_signal(|| None::<String>);
    let mut busy = use_signal(|| false);

    // Se suscribe a la sesión: al renovar el token se recarga el perfil.
    let mut me = use_resource(move || {
        let token = auth.read().session.clone().map(|s| s.access_token);
        async move {
            match token {
                Some(t) => api::me(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    // Resumen CRM del negocio.
    let token_crm = auth.read().session.clone().map(|s| s.access_token);
    let token_top = token_crm.clone();
    let mut stats = use_resource(move || {
        let token = token_crm.clone();
        async move {
            match token {
                Some(t) => api::dashboard_stats(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let top = use_resource(move || {        let token = token_top.clone();
        async move {
            match token {
                Some(t) => api::dashboard_top_deals(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Panel".to_string(),
                div { class: "alert alert-info",
                    p { "Esta página requiere sesión." }
                    Link {
                        to: Route::Login {},
                        class: "btn btn-small btn-primary",
                        "Iniciar sesión"
                    }
                }
            }
        };
    }

    rsx! {
        Layout { title: "Panel".to_string(),
            if let Some(msg) = action_error.read().clone() {
                div { class: "alert alert-error", "{msg}" }
            }
            match &*me.read() {
                None => rsx! {
                    p { class: "muted", "Cargando tu perfil…" }
                },
                Some(Ok(user)) => {
                    let refresh_token = auth
                        .read()
                        .session
                        .clone()
                        .map(|s| s.refresh_token)
                        .unwrap_or_default();
                    let perms = if user.permissions.is_empty() {
                        "—".to_string()
                    } else {
                        user.permissions.join(", ")
                    };
                    rsx! {
                        section { class: "card",
                            h2 { class: "card-title", "Tu perfil" }
                            dl { class: "detail-list",
                                div {
                                    dt { "Nombre" }
                                    dd { "{user.name}" }
                                }
                                div {
                                    dt { "Email" }
                                    dd { "{user.email}" }
                                }
                                if let Some(first) = user.first_name.clone() {
                                    div {
                                        dt { "Nombres" }
                                        dd { "{first}" }
                                    }
                                }
                                if let Some(last) = user.last_name.clone() {
                                    div {
                                        dt { "Apellidos" }
                                        dd { "{last}" }
                                    }
                                }
                                div {
                                    dt { "Permisos" }
                                    dd { class: "muted small", "{perms}" }
                                }
                            }
                            EditProfile { on_saved: move |_| me.restart() }
                            div { class: "row",
                                button {
                                    class: "btn btn-small",
                                    disabled: *busy.read(),
                                    onclick: move |_| {
                                        let refresh_v = refresh_token.clone();
                                        action_error.set(None);
                                        busy.set(true);
                                        spawn(async move {
                                            match api::refresh(&refresh_v).await {
                                                Ok(pair) => {
                                                    auth.write().session = Some(Session::from_pair(pair));
                                                    me.restart();
                                                }
                                                Err(err) => action_error.set(Some(err.to_string())),
                                            }
                                            busy.set(false);
                                        });
                                    },
                                    if *busy.read() {
                                        "Renovando…"
                                    } else {
                                        "Renovar tokens"
                                    }
                                }
                                button { class: "btn btn-small", onclick: move |_| me.restart(), "Recargar perfil" }
                                Link { to: Route::Users {}, class: "btn btn-small", "Ver usuarios" }
                            }
                            p { class: "muted small",
                                "Renovar rota el refresh token en el backend: el anterior queda revocado."
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar tu perfil." }
                        p { class: "muted small", "{e}" }
                        div { class: "row",
                            button { class: "btn btn-small", onclick: move |_| me.restart(), "Reintentar" }
                            button {
                                class: "btn btn-small",
                                onclick: move |_| {
                                    auth.write().session = None;
                                    nav.push(Route::Login {});
                                },
                                "Cerrar sesión local"
                            }
                        }
                    }
                },
            }
            section { class: "card",
                h2 { class: "card-title", "El negocio hoy" }
                match &*stats.read() {
                    None => rsx! { p { class: "muted", "Cargando resumen…" } },
                    Some(Err(e)) => rsx! {
                        p { class: "muted small",
                            "Sin resumen (¿falta permiso?): {e} "
                            button { class: "btn btn-small", onclick: move |_| stats.restart(), "Reintentar" }
                        }
                    },
                    Some(Ok(s)) => {
                        let revenue = format!("{:.0}", s.total_revenue);
                        rsx! {
                            div { class: "grid-cards",
                                div { class: "card",
                                    h2 { class: "card-title", "Contactos" }
                                    p { "{s.total_contacts}" }
                                }
                                div { class: "card",
                                    h2 { class: "card-title", "Negocios activos" }
                                    p { "{s.active_deals}" }
                                }
                                div { class: "card",
                                    h2 { class: "card-title", "Ganados / Perdidos" }
                                    p { "{s.won_deals} / {s.lost_deals}" }
                                }
                                div { class: "card",
                                    h2 { class: "card-title", "Ingresos (cerrados)" }
                                    p { "{revenue}" }
                                }
                            }
                        }
                    },
                }
            }
            section { class: "card",
                h2 { class: "card-title", "Mejores negocios" }
                match &*top.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                    Some(Ok(t)) => rsx! {
                        if t.deals.is_empty() {
                            p { class: "muted", "Aún no hay negocios." }
                        } else {
                            ul {
                                for d in &t.deals {
                                    {
                                        let did = d.id.to_string();
                                        let title = d.title.clone();
                                        let st = api::deal_stage_label(&d.stage).to_string();
                                        let amount = format!("{:.0}", d.value);
                                        rsx! {
                                            li { key: "{did}",
                                                Link {
                                                    to: Route::DealDetail { id: did },
                                                    class: "link",
                                                    "{title}"
                                                }
                                                span { class: "muted small", " · {st} · {amount}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
                div { class: "row",
                    Link { to: Route::Contacts {}, class: "btn btn-small", "Ir a contactos" }
                    Link { to: Route::Deals {}, class: "btn btn-small btn-primary", "Ir al tablero" }
                }
            }
        }
    }
}

/// Rectificación directa: edita nombre/email vía `PUT /api/v1/auth/profile`.
/// Al guardar, avisa al padre para recargar y actualiza la sesión local.
#[component]
fn EditProfile(on_saved: EventHandler<()>) -> Element {
    let mut auth = use_auth();
    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut ok = use_signal(|| false);

    rsx! {
        form {
            class: "form",
            onsubmit: move |e| {
                e.prevent_default();
                let name_v = name.read().trim().to_string();
                let email_v = email.read().trim().to_lowercase();
                if name_v.is_empty() && email_v.is_empty() {
                    error.set(Some("Cambia tu nombre o tu email para guardar.".to_string()));
                    return;
                }
                let token = auth
                    .read()
                    .session
                    .clone()
                    .map(|s| s.access_token)
                    .unwrap_or_default();
                busy.set(true);
                error.set(None);
                ok.set(false);
                spawn(async move {
                    let email_opt = if email_v.is_empty() {
                        None
                    } else {
                        Some(email_v.as_str())
                    };
                    let name_opt = if name_v.is_empty() { None } else { Some(name_v.as_str()) };
                    match api::update_profile(&token, email_opt, name_opt).await {
                        Ok(updated) => {
                            let current = auth.read().session.clone();
                            if let Some(mut session) = current {
                                session.user.email = updated.email.clone();
                                session.user.name = updated.name.clone();
                                auth.write().session = Some(session);
                            }
                            ok.set(true);
                            on_saved.call(());
                        }
                        Err(err) => error.set(Some(err.to_string())),
                    }
                    busy.set(false);
                });
            },
            h3 { class: "card-title", "Rectificar mis datos" }
            if let Some(msg) = error.read().clone() {
                div { class: "alert alert-error", "{msg}" }
            }
            if *ok.read() {
                div { class: "alert alert-ok", "Datos actualizados." }
            }
            label { class: "field", r#for: "edit-name", "Nombre" }
            input {
                id: "edit-name",
                class: "input",
                r#type: "text",
                value: "{name}",
                oninput: move |e| name.set(e.value()),
                maxlength: 100,
                autocomplete: "name",
            }
            label { class: "field", r#for: "edit-email", "Email" }
            input {
                id: "edit-email",
                class: "input",
                r#type: "email",
                value: "{email}",
                oninput: move |e| email.set(e.value()),
                autocomplete: "email",
            }
            button {
                class: "btn btn-small",
                r#type: "submit",
                disabled: *busy.read(),
                if *busy.read() {
                    "Guardando…"
                } else {
                    "Guardar cambios"
                }
            }
        }
    }
}
