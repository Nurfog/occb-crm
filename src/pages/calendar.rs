//! Calendario: eventos locales, estado de conexión y sincronización.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Abre una URL en pestaña nueva (solo navegador).
fn open_url(url: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(w) = web_sys::window() {
            let _ = w.open_with_url_and_target(url, "_blank");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = url;
    }
}

/// Eventos, conexión OAuth y alta. Requiere sesión.
#[component]
pub fn Calendar() -> Element {
    let auth = use_auth();
    let mut title = use_signal(String::new);
    let mut start = use_signal(String::new);
    let mut end = use_signal(String::new);
    let mut error = use_signal(|| None::<String>);
    let mut sync_msg = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_e = token_of.clone();
    let token_cn = token_of.clone();
    let token_sy = token_of.clone();
    let token_cr = token_of.clone();
    let token_del = token_of.clone();
    let mut events = use_resource(move || {
        let token = token_e.clone();
        async move {
            match token {
                Some(t) => api::list_calendar_events(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_s = token_of.clone();
    let mut status = use_resource(move || {
        let token = token_s.clone();
        async move {
            match token {
                Some(t) => api::calendar_status(&t).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Calendario".to_string(),
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
        Layout { title: "Calendario".to_string(),
            section { class: "card",
                h2 { class: "card-title", "Conexiones" }
                match &*status.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo ver: {e}" } },
                    Some(Ok(s)) => {
                        let g = if s.google { "conectado" } else { "no conectado" };
                        let m = if s.microsoft { "conectado" } else { "no conectado" };
                        rsx! {
                            p { "Google: {g} · Microsoft: {m}" }
                            div { class: "row",
                                for p in ["google", "microsoft"] {
                                    {
                                        let tok = token_cn.clone();
                                        rsx! {
                                            button {
                                                key: "{p}",
                                                class: "btn btn-small",
                                                onclick: move |_| {
                                                    let tk = tok.clone();
                                                    let pv = p.to_string();
                                                    spawn(async move {
                                                        if let Some(t) = tk {
                                                            match api::calendar_auth_url(&t, &pv).await {
                                                                Ok(u) => {
                                                                    let link = if u.url.is_empty() {
                                                                        u.auth_url.clone()
                                                                    } else {
                                                                        u.url.clone()
                                                                    };
                                                                    open_url(&link);
                                                                }
                                                                Err(e) => sync_msg.set(Some(e.to_string())),
                                                            }
                                                        }
                                                    });
                                                },
                                                "Conectar {p}"
                                            }
                                        }
                                    }
                                }
                                button {
                                    class: "btn btn-small",
                                    onclick: move |_| {
                                        let tok = token_sy.clone();
                                        sync_msg.set(None);
                                        spawn(async move {
                                            if let Some(t) = tok {
                                                match api::sync_google_calendar(&t).await {
                                                    Ok(r) => {
                                                        sync_msg.set(
                                                            Some(format!("Sincronizados: {}.", r.synced)),
                                                        );
                                                        events.restart();
                                                    }
                                                    Err(e) => sync_msg.set(Some(e.to_string())),
                                                }
                                            }
                                        });
                                    },
                                    "Sincronizar Google"
                                }
                            }
                            if let Some(msg) = sync_msg.read().clone() {
                                p { class: "alert alert-info", "{msg}" }
                            }
                        }
                    },
                }
            }
            section { class: "card form",
                h2 { class: "card-title", "Nuevo evento" }
                label { class: "field", "Título *" }
                input {
                    class: "input",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                }
                div { class: "row",
                    div {
                        label { class: "field", "Inicio (AAAA-MM-DDTHH:MM)" }
                        input {
                            class: "input",
                            placeholder: "2026-10-20T10:00",
                            value: "{start}",
                            oninput: move |e| start.set(e.value()),
                        }
                    }
                    div {
                        label { class: "field", "Fin (AAAA-MM-DDTHH:MM)" }
                        input {
                            class: "input",
                            placeholder: "2026-10-20T11:00",
                            value: "{end}",
                            oninput: move |e| end.set(e.value()),
                        }
                    }
                }
                if let Some(msg) = error.read().clone() {
                    p { class: "alert alert-error", "{msg}" }
                }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| {
                        let tok = token_cr.clone();
                        let t = title.read().trim().to_string();
                        let s = start.read().trim().replace(' ', "T");
                        let e = end.read().trim().replace(' ', "T");
                        if t.is_empty() || s.is_empty() || e.is_empty() {
                            error.set(Some("Título, inicio y fin son obligatorios.".to_string()));
                            return;
                        }
                        if chrono::DateTime::parse_from_rfc3339(&to_rfc3339(&s)).is_err() {
                            error.set(Some("Revisa el formato de fecha (AAAA-MM-DDTHH:MM).".to_string()));
                            return;
                        }
                        error.set(None);
                        spawn(async move {
                            if let Some(tk) = tok {
                                match api::create_calendar_event(
                                        &tk,
                                        &api::CreateCalendarEventBody {
                                            title: &t,
                                            description: None,
                                            location: None,
                                            start_time: &to_rfc3339(&s),
                                            end_time: &to_rfc3339(&e),
                                            all_day: None,
                                        },
                                    )
                                    .await
                                {
                                    Ok(_) => {
                                        title.set(String::new());
                                        start.set(String::new());
                                        end.set(String::new());
                                        events.restart();
                                    }
                                    Err(err) => error.set(Some(err.to_string())),
                                }
                            }
                        });
                    },
                    "Crear evento"
                }
                p { class: "muted small", "Los eventos creados aquí son locales (provider local)." }
            }
            section { class: "card",
                h2 { class: "card-title", "Próximos eventos" }
                match &*events.read() {
                    None => rsx! { p { class: "muted", "Cargando…" } },
                    Some(Err(e)) => rsx! {
                        div { class: "alert alert-error",
                            p { "No se pudieron cargar." }
                            p { class: "muted small", "{e}" }
                            button { class: "btn btn-small", onclick: move |_| events.restart(), "Reintentar" }
                        }
                    },
                    Some(Ok(list)) => rsx! {
                        if list.is_empty() {
                            p { class: "muted", "Sin eventos." }
                        } else {
                            ul {
                                for ev in list {
                                    {
                                        let key = ev.id.to_string();
                                        let id_v = key.clone();
                                        let etitle = ev.title.clone();
                                        let when = format!(
                                            "{} → {}",
                                            ev.start_time.format("%Y-%m-%d %H:%M"),
                                            ev.end_time.format("%H:%M")
                                        );
                                        let prov = ev.provider.clone();
                                        let tok = token_del.clone();
                                        rsx! {
                                            li { key: "{key}",
                                                strong { "{etitle}" }
                                                span { class: "muted small", " · {when} · {prov} " }
                                                button {
                                                    class: "link-btn small",
                                                    onclick: move |_| {
                                                        let tk = tok.clone();
                                                        let iv = id_v.clone();
                                                        spawn(async move {
                                                            if let Some(t) = tk {
                                                                let _ = api::delete_calendar_event(&t, &iv).await;
                                                            }
                                                            events.restart();
                                                        });
                                                    },
                                                    "eliminar"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Normaliza `AAAA-MM-DDTHH:MM` (o con segundos) a RFC3339 con `Z`.
fn to_rfc3339(s: &str) -> String {
    let with_secs = if s.len() == 16 { format!("{s}:00") } else { s.to_string() };
    format!("{with_secs}Z")
}
