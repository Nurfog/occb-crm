//! Calendario: eventos locales, estado de conexión y sincronización.

use dioxus::prelude::*;
use chrono::Datelike as _;

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

/// Resta/suma meses a un `AAAA-MM`.
fn shift_cal_month(month: &mut Signal<String>, delta: i32) {
    let cur = month.read().clone();
    let (mut y, mut m): (i32, u32) = cur
        .split_once('-')
        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        .unwrap_or((2026, 1));
    m = (m as i32 + delta) as u32;
    while m < 1 {
        m += 12;
        y -= 1;
    }
    while m > 12 {
        m -= 12;
        y += 1;
    }
    month.set(format!("{y:04}-{m:02}"));
}

/// Mes con los eventos del calendario.
#[component]
fn CalendarMonth(
    year_month: String,
    events: Vec<api::CalendarEvent>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
    on_today: EventHandler<()>,
) -> Element {
    use chrono::{Datelike, NaiveDate};

    let (y, m): (i32, u32) = year_month
        .split_once('-')
        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        .unwrap_or((2026, 1));
    let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap_or(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap());
    let lead = first.weekday().num_days_from_monday() as usize;
    let next_first = if m == 12 {
        NaiveDate::from_ymd_opt(y + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(y, m + 1, 1).unwrap()
    };
    let days = (next_first - first).num_days() as u32;
    let title = first.format("%B %Y").to_string();
    let today = chrono::Utc::now().date_naive().format("%Y-%m-%d").to_string();

    let mut by_day: std::collections::HashMap<String, Vec<api::CalendarEvent>> =
        std::collections::HashMap::new();
    for e in &events {
        by_day
            .entry(e.start_time.date_naive().format("%Y-%m-%d").to_string())
            .or_default()
            .push(e.clone());
    }

    let mut cells: Vec<Element> = Vec::new();
    for _ in 0..lead {
        cells.push(rsx! { div { class: "cal-day cal-empty" } });
    }
    for d in 1..=days {
        let key = format!("{y:04}-{m:02}-{d:02}");
        let is_today = key == today;
        let evts = by_day.get(&key).cloned().unwrap_or_default();
        let shown: Vec<api::CalendarEvent> = evts.iter().take(3).cloned().collect();
        let more = evts.len().saturating_sub(3);
        let day_cls = if is_today { "cal-day cal-today" } else { "cal-day" };
        cells.push(rsx! {
            div { class: "{day_cls}", key: "{key}",
                span { class: "cal-num", "{d}" }
                for e in shown {
                    {
                        let title = e.title.clone();
                        let when = e.start_time.format("%H:%M").to_string();
                        rsx! {
                            div { class: "cal-evt", key: "{title}-{when}", "{when} {title}" }
                        }
                    }
                }
                if more > 0 {
                    span { class: "muted small", "+{more}" }
                }
            }
        });
    }

    rsx! {
        section { class: "card",
            div { class: "row cal-head",
                button { class: "btn btn-small", onclick: move |_| on_prev.call(()), "‹" }
                button { class: "btn btn-small", onclick: move |_| on_today.call(()), "Hoy" }
                button { class: "btn btn-small", onclick: move |_| on_next.call(()), "›" }
                h2 { class: "card-title cal-title", "{title}" }
            }
            div { class: "cal-grid",
                for w in ["Lun", "Mar", "Mié", "Jue", "Vie", "Sáb", "Dom"] {
                    div { class: "cal-dow", key: "{w}", "{w}" }
                }
                for cell in cells {
                    {cell}
                }
            }
        }
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
    let mut month = use_signal(|| {
        let now = chrono::Utc::now().date_naive();
        format!("{:04}-{:02}", now.year(), now.month())
    });

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
            match &*events.read() {
                Some(Ok(list)) => rsx! {
                    CalendarMonth {
                        year_month: month.read().clone(),
                        events: list.clone(),
                        on_prev: move |_| shift_cal_month(&mut month, -1),
                        on_next: move |_| shift_cal_month(&mut month, 1),
                        on_today: move |_| {
                            let now = chrono::Utc::now().date_naive();
                            month.set(format!("{:04}-{:02}", now.year(), now.month()));
                        },
                    }
                },
                _ => rsx! {},
            }
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
