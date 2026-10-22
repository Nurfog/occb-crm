//! Actividades: listado, alta, completar y borrado (`/api/v1/activities`).

use dioxus::prelude::*;
use chrono::Datelike as _;

use crate::{
    api::{self, ApiError, ACTIVITY_TYPES},
    auth::use_auth,
    components::Layout,
    route::Route,
};

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Listado con filtro por tipo + alta. Requiere sesión.
#[component]
pub fn Activities() -> Element {
    let auth = use_auth();
    let mut kind = use_signal(String::new);
    let mut show_new = use_signal(|| false);
    let mut view = use_signal(|| "mes".to_string());
    let mut month = use_signal(|| {
        let now = chrono::Utc::now().date_naive();
        format!("{:04}-{:02}", now.year(), now.month())
    });
    let mut selected = use_signal(|| None::<api::Activity>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut items = use_resource(move || {
        let token = token_of.clone();
        let k = kind.read().clone();
        async move {
            match token {
                Some(t) => api::list_activities(&t, &k, "", "", "").await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Actividades".to_string(),
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

    let creating = *show_new.read();
    let new_label = if creating { "Cerrar formulario" } else { "+ Nueva actividad" };
    let is_month = *view.read() == "mes";

    rsx! {
        Layout { title: "Actividades".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
                button {
                    class: if is_month { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| view.set("mes".to_string()),
                    "Mes"
                }
                button {
                    class: if !is_month { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| view.set("lista".to_string()),
                    "Lista"
                }
                if !is_month {
                    select {
                        class: "input",
                        value: "{kind}",
                        onchange: move |e| {
                            kind.set(e.value());
                            items.restart();
                        },
                        option { value: "", "Todos los tipos" }
                        for s in ACTIVITY_TYPES {
                            {
                                let label = api::activity_type_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                }
            }
            if creating {
                NewActivityForm {
                    initial_due: "",
                    on_created: move || {
                        show_new.set(false);
                        items.restart();
                    },
                }
            }
            if let Some(sel) = selected.read().clone() {
                ActivityDetail {
                    activity: sel,
                    on_close: move || selected.set(None),
                    on_changed: move || {
                        selected.set(None);
                        items.restart();
                    },
                }
            }
            if is_month {
                match &*items.read() {
                    None => rsx! { p { class: "muted", "Cargando calendario…" } },
                    Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                    Some(Ok(list)) => rsx! {
                        MonthView {
                            year_month: month.read().clone(),
                            activities: list.clone(),
                            on_prev: move |_| shift_month(&mut month, -1),
                            on_next: move |_| shift_month(&mut month, 1),
                            on_today: move |_| {
                                let now = chrono::Utc::now().date_naive();
                                month.set(format!("{:04}-{:02}", now.year(), now.month()));
                            },
                            on_pick: move |a: api::Activity| selected.set(Some(a)),
                        }
                    },
                }
            } else {
            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando actividades…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver actividades."
                            } else {
                                "No se pudieron cargar."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
                    }
                },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin actividades." }
                    } else {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Asunto" }
                                        th { "Tipo" }
                                        th { "Vence" }
                                        th { "Estado" }
                                        th { "Acciones" }
                                    }
                                }
                                tbody {
                                    for a in list {
                                        {
                                            let key = a.id.to_string();
                                            let subj = a.subject.clone();
                                            let label = api::activity_type_label(&a.activity_type).to_string();
                                            let due = a
                                                .due_date
                                                .map(|d| d.format("%Y-%m-%d").to_string())
                                                .unwrap_or_else(|| "—".to_string());
                                            let state = if a.completed { "Hecha ✓" } else { "Pendiente" };
                                            let show_done = !a.completed;
                                            let key_done = key.clone();
                                            let key_del = key.clone();
                                            let token_base = auth
                                                .read()
                                                .session
                                                .clone()
                                                .map(|s| s.access_token);
                                            let token_done = token_base.clone();
                                            let token_del = token_base.clone();
                                            rsx! {
                                                tr { key: "{key}",
                                                    td { "{subj}" }
                                                    td { "{label}" }
                                                    td { class: "muted", "{due}" }
                                                    td { "{state}" }
                                                    td {
                                                        if show_done {
                                                            button {
                                                                class: "btn btn-small",
                                                                onclick: move |_| {
                                                                    let token_v = token_done.clone();
                                                                    let id_c = key_done.clone();
                                                                    spawn(async move {
                                                                        if let Some(t) = token_v {
                                                                            let _ = api::complete_activity(&t, &id_c).await;
                                                                        }
                                                                        items.restart();
                                                                    });
                                                                },
                                                                "Marcar hecha"
                                                            }
                                                        }
                                                        button {
                                                            class: "btn btn-small",
                                                            onclick: move |_| {
                                                                let token_v = token_del.clone();
                                                                let id_c = key_del.clone();
                                                                spawn(async move {
                                                                    if let Some(t) = token_v {
                                                                        let _ = api::delete_activity(&t, &id_c).await;
                                                                    }
                                                                    items.restart();
                                                                });
                                                            },
                                                            "Eliminar"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
            } // fin vista lista
        }
    }
}

/// Resta/suma meses a un `AAAA-MM`.
fn shift_month(month: &mut Signal<String>, delta: i32) {
    let cur = month.read().clone();
    let (mut y, mut m): (i32, i32) = cur
        .split_once('-')
        .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
        .unwrap_or((2026, 1));
    m += delta;
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

/// Vista mensual de actividades (como el original).
#[component]
fn MonthView(
    year_month: String,
    activities: Vec<api::Activity>,
    on_prev: EventHandler<()>,
    on_next: EventHandler<()>,
    on_today: EventHandler<()>,
    on_pick: EventHandler<api::Activity>,
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

    let mut by_day: std::collections::HashMap<String, Vec<api::Activity>> =
        std::collections::HashMap::new();
    for a in &activities {
        if let Some(d) = a.due_date {
            by_day
                .entry(d.date_naive().format("%Y-%m-%d").to_string())
                .or_default()
                .push(a.clone());
        }
    }

    let mut cells: Vec<Element> = Vec::new();
    for _ in 0..lead {
        cells.push(rsx! { div { class: "cal-day cal-empty" } });
    }
    for d in 1..=days {
        let key = format!("{y:04}-{m:02}-{d:02}");
        let is_today = key == today;
        let evts = by_day.get(&key).cloned().unwrap_or_default();
        let shown: Vec<api::Activity> = evts.iter().take(3).cloned().collect();
        let more = evts.len().saturating_sub(3);
        let day_cls = if is_today { "cal-day cal-today" } else { "cal-day" };
        cells.push(rsx! {
            div { class: "{day_cls}", key: "{key}",
                span { class: "cal-num", "{d}" }
                for e in shown {
                    {
                        let title = e.subject.clone();
                        rsx! {
                            button {
                                class: "cal-evt",
                                onclick: move |_| on_pick.call(e.clone()),
                                "{title}"
                            }
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

/// Detalle de una actividad con completar/borrar.
#[component]
fn ActivityDetail(
    activity: api::Activity,
    on_close: EventHandler<()>,
    on_changed: EventHandler<()>,
) -> Element {
    let auth = use_auth();
    let mut busy = use_signal(|| false);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_done = token_of.clone();
    let token_del = token_of.clone();
    let id = activity.id.to_string();
    let id_done = id.clone();
    let id_del = id.clone();
    let state = if activity.completed { "Hecha ✓" } else { "Pendiente" };

    rsx! {
        section { class: "card",
            div { class: "row",
                h2 { class: "card-title", "{activity.subject}" }
                button {
                    class: "btn btn-small",
                    onclick: move |_| on_close.call(()),
                    "Cerrar"
                }
            }
            p { class: "muted", "{activity.activity_type} · {state}" }
            if let Some(desc) = activity.description.clone() {
                p { "{desc}" }
            }
            if let Some(due) = activity.due_date {
                {
                    let when = due.format("%d %b %Y %H:%M").to_string();
                    rsx! {
                        p { class: "muted small", "Vence: {when}" }
                    }
                }
            }
            div { class: "row",
                if !activity.completed {
                    button {
                        class: "btn btn-small btn-primary",
                        disabled: *busy.read(),
                        onclick: move |_| {
                            let tok = token_done.clone();
                            let iv = id_done.clone();
                            busy.set(true);
                            spawn(async move {
                                if let Some(t) = tok {
                                    let _ = api::complete_activity(&t, &iv).await;
                                }
                                busy.set(false);
                                on_changed.call(());
                            });
                        },
                        "Marcar hecha"
                    }
                }
                button {
                    class: "btn btn-small danger-btn",
                    disabled: *busy.read(),
                    onclick: move |_| {
                        let tok = token_del.clone();
                        let iv = id_del.clone();
                        busy.set(true);
                        spawn(async move {
                            if let Some(t) = tok {
                                let _ = api::delete_activity(&t, &iv).await;
                            }
                            busy.set(false);
                            on_changed.call(());
                        });
                    },
                    "Eliminar"
                }
            }
        }
    }
}

/// Alta de actividad.
#[component]
fn NewActivityForm(initial_due: String, on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut kind = use_signal(|| "task".to_string());
    let mut subject = use_signal(String::new);
    let mut due = use_signal(|| initial_due.clone());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear actividad" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let k = kind.read().clone();
                let subj = subject.read().trim().to_string();
                if subj.is_empty() {
                    error.set(Some("El asunto es obligatorio.".to_string()));
                    return;
                }
                let due_v = none_if_empty(due.read().trim());
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::create_activity(
                                    &t,
                                    &api::CreateActivityBody {
                                        activity_type: &k,
                                        subject: &subj,
                                        description: None,
                                        contact_id: None,
                                        deal_id: None,
                                        company_id: None,
                                        due_date: due_v.as_deref(),
                                    },
                                )
                                .await
                                .map(|_| ())
                                .map_err(|e| e.to_string())
                        }
                        None => Err("Sin sesión.".to_string()),
                    };
                    busy.set(false);
                    match res {
                        Ok(()) => on_created.call(()),
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "Nueva actividad" }
            div { class: "row",
                div {
                    label { class: "field", "Tipo" }
                    select {
                        class: "input",
                        value: "{kind}",
                        onchange: move |e| kind.set(e.value()),
                        for s in ACTIVITY_TYPES {
                            {
                                let label = api::activity_type_label(s).to_string();
                                rsx! { option { value: s, "{label}" } }
                            }
                        }
                    }
                }
                div {
                    label { class: "field", "Vence (AAAA-MM-DD)" }
                    input {
                        class: "input",
                        placeholder: "2026-12-31",
                        value: "{due}",
                        oninput: move |e| due.set(e.value()),
                    }
                }
            }
            label { class: "field", "Asunto *" }
            input {
                class: "input",
                value: "{subject}",
                oninput: move |e| subject.set(e.value()),
                required: true,
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            button {
                class: "btn btn-primary",
                r#type: "submit",
                disabled: busy_v,
                "{submit_label}"
            }
        }
    }
}
