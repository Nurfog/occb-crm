//! Contactos: listado, detalle, alta, edición y borrado (`/api/v1/contacts`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::{tags::EntityTags, Layout},
    route::Route,
};

const SORT_OPTIONS: [(&str, &str); 5] = [
    ("created_at", "Recientes"),
    ("first_name", "Nombre"),
    ("last_name", "Apellido"),
    ("email", "Email"),
    ("position", "Cargo"),
];

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Iniciales para el avatar (hasta 2 letras).
fn contact_initials(first: &str, last: &str) -> String {
    let mut out = String::new();
    if let Some(c) = first.chars().next() {
        out.push(c.to_ascii_uppercase());
    }
    if let Some(c) = last.chars().next() {
        out.push(c.to_ascii_uppercase());
    }
    if out.is_empty() { "·".to_string() } else { out }
}

/// Alterna el orden de una columna (desc → asc → desc) y vuelve a la página 1.
fn toggle_sort(
    sort_by: &mut Signal<String>,
    order: &mut Signal<String>,
    page: &mut Signal<u32>,
    col: &str,
) {
    if *sort_by.read() == col {
        order.set(if *order.read() == "asc" { "desc".to_string() } else { "asc".to_string() });
    } else {
        sort_by.set(col.to_string());
        order.set("desc".to_string());
    }
    page.set(1);
}

/// Listado con búsqueda, orden y paginación + alta. Requiere sesión.
#[component]
pub fn Contacts() -> Element {
    let auth = use_auth();
    let mut search = use_signal(String::new);
    let mut applied = use_signal(String::new);
    let mut page = use_signal(|| 1u32);
    let per_page = use_signal(|| 20u32);
    let mut sort_by = use_signal(|| "created_at".to_string());
    let mut order = use_signal(|| "desc".to_string());
    let mut show_new = use_signal(|| false);
    let mut selected = use_signal(Vec::<String>::new);
    let mut op_msg = use_signal(|| None::<String>);
    let mut show_filters = use_signal(|| false);
    let mut f_company = use_signal(String::new);
    let mut f_position = use_signal(String::new);
    let mut f_after = use_signal(String::new);
    let mut f_before = use_signal(String::new);
    let mut applied_company = use_signal(String::new);
    let mut applied_position = use_signal(String::new);
    let mut applied_after = use_signal(String::new);
    let mut applied_before = use_signal(String::new);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_ex = token_of.clone();
    let token_bk = token_of.clone();
    let token_im = token_of.clone();
    let mut contacts = use_resource(move || {
        let token = token_of.clone();
        let q = applied.read().clone();
        let p = *page.read();
        let pp = *per_page.read();
        let sort_v = sort_by.read().clone();
        let order_v = order.read().clone();
        let co = applied_company.read().clone();
        let po = applied_position.read().clone();
        let af = applied_after.read().clone();
        let be = applied_before.read().clone();
        async move {
            match token {
                Some(t) => {
                    api::list_contacts(&t, p, pp, &q, &sort_v, &order_v, &co, &po, &af, &be).await
                }
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Contactos".to_string(),
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
    let new_label = if creating { "Cerrar formulario" } else { "+ Nuevo contacto" };
    let n_selected = selected.read().len();
    let filters_open = *show_filters.read();

    rsx! {
        Layout { title: "Contactos".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_ex.clone();
                        let qq = applied.read().clone();
                        spawn(async move {
                            if let Some(t) = tok {
                                match api::export_contacts(&t, &qq).await {
                                    Ok(csv) => {
                                        let _ = crate::privacy::download_text_file(
                                            "contactos.csv",
                                            &csv,
                                        );
                                    }
                                    Err(e) => op_msg.set(Some(e.to_string())),
                                }
                            }
                        });
                    },
                    "Exportar CSV"
                }
                if n_selected > 0 {
                    button {
                        class: "btn btn-small",
                        onclick: move |_| {
                            let tok = token_bk.clone();
                            let ids = selected.read().clone();
                            op_msg.set(None);
                            spawn(async move {
                                if let Some(t) = tok {
                                    match api::bulk_delete_contacts(&t, &ids).await {
                                        Ok(r) => {
                                            op_msg.set(
                                                Some(format!("Eliminados: {}.", r.deleted)),
                                            );
                                            selected.set(Vec::new());
                                            contacts.restart();
                                        }
                                        Err(e) => op_msg.set(Some(e.to_string())),
                                    }
                                }
                            });
                        },
                        "Eliminar {n_selected} seleccionados"
                    }
                }
            }
            div { class: "row",
                input {
                    r#type: "file",
                    id: "contacts-import",
                    accept: ".csv,text/csv",
                }
                button {
                    class: "btn btn-small",
                    onclick: move |_| {
                        let tok = token_im.clone();
                        op_msg.set(None);
                        spawn(async move {
                            match crate::files::read_first_file("contacts-import").await {
                                Ok((_, _, bytes)) => match String::from_utf8(bytes) {
                                    Ok(csv) => {
                                        if let Some(t) = tok {
                                            match api::import_contacts(&t, &csv).await {
                                                Ok(r) => {
                                                    op_msg.set(
                                                        Some(
                                                            format!(
                                                                "Importados: {}. Filas con error: {}.",
                                                                r.imported,
                                                                r.errors.len()
                                                            ),
                                                        ),
                                                    );
                                                    contacts.restart();
                                                }
                                                Err(e) => op_msg.set(Some(e.to_string())),
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        op_msg.set(
                                            Some("El archivo no es UTF-8 válido.".to_string()),
                                        );
                                    }
                                },
                                Err(e) => op_msg.set(Some(e)),
                            }
                        });
                    },
                    "Importar CSV"
                }
            }
            if let Some(msg) = op_msg.read().clone() {
                p { class: "alert alert-info", "{msg}" }
            }
            if creating {
                NewContactForm { on_created: move || {
                    show_new.set(false);
                    contacts.restart();
                } }
            }
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied.set(search.read().clone());
                    page.set(1);
                },
                input {
                    class: "input",
                    r#type: "search",
                    placeholder: "Buscar por nombre o email…",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
                button {
                    class: "btn btn-small",
                    r#type: "button",
                    onclick: move |_| show_filters.set(!filters_open),
                    if filters_open { "Ocultar filtros" } else { "Filtros" }
                }
            }
            if filters_open {
                form {
                    class: "card form",
                    onsubmit: move |e| {
                        e.prevent_default();
                        applied_company.set(f_company.read().clone());
                        applied_position.set(f_position.read().clone());
                        applied_after.set(f_after.read().trim().to_string());
                        applied_before.set(f_before.read().trim().to_string());
                        page.set(1);
                    },
                    div { class: "row",
                        div {
                            label { class: "field", "Empresa" }
                            input {
                                class: "input",
                                value: "{f_company}",
                                oninput: move |e| f_company.set(e.value()),
                            }
                        }
                        div {
                            label { class: "field", "Cargo" }
                            input {
                                class: "input",
                                value: "{f_position}",
                                oninput: move |e| f_position.set(e.value()),
                            }
                        }
                    }
                    div { class: "row",
                        div {
                            label { class: "field", "Creado desde" }
                            input {
                                class: "input",
                                r#type: "date",
                                value: "{f_after}",
                                oninput: move |e| f_after.set(e.value()),
                            }
                        }
                        div {
                            label { class: "field", "Creado hasta" }
                            input {
                                class: "input",
                                r#type: "date",
                                value: "{f_before}",
                                oninput: move |e| f_before.set(e.value()),
                            }
                        }
                    }
                    div { class: "row",
                        button { class: "btn btn-small btn-primary", r#type: "submit", "Aplicar" }
                        button {
                            class: "btn btn-small",
                            r#type: "button",
                            onclick: move |_| {
                                f_company.set(String::new());
                                f_position.set(String::new());
                                f_after.set(String::new());
                                f_before.set(String::new());
                                applied_company.set(String::new());
                                applied_position.set(String::new());
                                applied_after.set(String::new());
                                applied_before.set(String::new());
                                page.set(1);
                            },
                            "Limpiar"
                        }
                    }
                }
            }

            match &*contacts.read() {
                None => rsx! { p { class: "muted", "Cargando contactos…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver contactos."
                            } else {
                                "No se pudo cargar el listado."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| contacts.restart(), "Reintentar" }
                    }
                },
                Some(Ok(paged)) => {
                    let total_pages = paged.total_pages.max(1);
                    let current = *page.read();
                    let total = paged.total;
                    let sort_v = sort_by.read().clone();
                    let order_v = order.read().clone();
                    let arrow = |col: &str| {
                        if sort_v == col {
                            if order_v == "asc" { " ▲" } else { " ▼" }
                        } else {
                            ""
                        }
                    };
                    rsx! {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "" }
                                        th {
                                            button {
                                                class: "link-btn th-sort",
                                                onclick: move |_| toggle_sort(&mut sort_by, &mut order, &mut page, "first_name"),
                                                "Nombre{arrow(\"first_name\")}"
                                            }
                                        }
                                        th {
                                            button {
                                                class: "link-btn th-sort",
                                                onclick: move |_| toggle_sort(&mut sort_by, &mut order, &mut page, "email"),
                                                "Email{arrow(\"email\")}"
                                            }
                                        }
                                        th {
                                            button {
                                                class: "link-btn th-sort",
                                                onclick: move |_| toggle_sort(&mut sort_by, &mut order, &mut page, "phone"),
                                                "Teléfono{arrow(\"phone\")}"
                                            }
                                        }
                                        th {
                                            button {
                                                class: "link-btn th-sort",
                                                onclick: move |_| toggle_sort(&mut sort_by, &mut order, &mut page, "position"),
                                                "Cargo{arrow(\"position\")}"
                                            }
                                        }
                                    }
                                }
                                tbody {
                                    for c in &paged.data {
                                        {
                                            let id = c.id.to_string();
                                            let is_sel = selected.read().contains(&id);
                                            let id_t = id.clone();
                                            let name = c.full_name();
                                            let initials = contact_initials(&c.first_name, &c.last_name);
                                            let email = c.email.clone().unwrap_or_default();
                                            let phone = c.phone.clone().unwrap_or_default();
                                            let position = c.position.clone().unwrap_or_default();
                                            rsx! {
                                                tr { key: "{id}",
                                                    td {
                                                        input {
                                                            r#type: "checkbox",
                                                            checked: is_sel,
                                                            onclick: move |_| {
                                                                let mut s = selected.read().clone();
                                                                if s.contains(&id_t) {
                                                                    s.retain(|x| x != &id_t);
                                                                } else {
                                                                    s.push(id_t.clone());
                                                                }
                                                                selected.set(s);
                                                            },
                                                        }
                                                    }
                                                    td {
                                                        span { class: "avatar", "{initials}" }
                                                        Link {
                                                            to: Route::ContactDetail { id: id.clone() },
                                                            class: "link",
                                                            "{name}"
                                                        }
                                                    }
                                                    td { "{email}" }
                                                    td { "{phone}" }
                                                    td { "{position}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if paged.data.is_empty() {
                                p { class: "muted", "Sin contactos. Crea el primero con «+ Nuevo contacto»." }
                            }
                        }
                        div { class: "pagination",
                            button {
                                class: "btn btn-small",
                                disabled: current <= 1,
                                onclick: move |_| page.set(current.saturating_sub(1).max(1)),
                                "← Anterior"
                            }
                            span { class: "muted small", "Página {current} de {total_pages} · {total} en total" }
                            button {
                                class: "btn btn-small",
                                disabled: current as i64 >= total_pages,
                                onclick: move |_| page.set(current + 1),
                                "Siguiente →"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Formulario de alta de contacto.
#[component]
fn NewContactForm(on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut first = use_signal(String::new);
    let mut last = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut position = use_signal(String::new);
    let mut notes = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear contacto" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let f = first.read().trim().to_string();
                let l = last.read().trim().to_string();
                if f.is_empty() || l.is_empty() {
                    error.set(Some("Nombre y apellido son obligatorios.".to_string()));
                    return;
                }
                let em = none_if_empty(email.read().trim());
                let ph = none_if_empty(phone.read().trim());
                let pos = none_if_empty(position.read().trim());
                let no = none_if_empty(notes.read().trim());
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::create_contact(
                                    &t,
                                    &api::CreateContactBody {
                                        first_name: &f,
                                        last_name: &l,
                                        email: em.as_deref(),
                                        phone: ph.as_deref(),
                                        company_id: None,
                                        position: pos.as_deref(),
                                        notes: no.as_deref(),
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
            h2 { class: "card-title", "Nuevo contacto" }
            div { class: "row",
                div {
                    label { class: "field", r#for: "nc-first", "Nombre *" }
                    input {
                        id: "nc-first",
                        class: "input",
                        value: "{first}",
                        oninput: move |e| first.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", r#for: "nc-last", "Apellido *" }
                    input {
                        id: "nc-last",
                        class: "input",
                        value: "{last}",
                        oninput: move |e| last.set(e.value()),
                        required: true,
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", r#for: "nc-email", "Email" }
                    input {
                        id: "nc-email",
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div {
                    label { class: "field", r#for: "nc-phone", "Teléfono" }
                    input {
                        id: "nc-phone",
                        class: "input",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }
            }
            label { class: "field", r#for: "nc-pos", "Cargo" }
            input {
                id: "nc-pos",
                class: "input",
                value: "{position}",
                oninput: move |e| position.set(e.value()),
            }
            label { class: "field", r#for: "nc-notes", "Notas" }
            textarea {
                id: "nc-notes",
                class: "input",
                rows: 2,
                value: "{notes}",
                oninput: move |e| notes.set(e.value()),
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

/// Detalle con edición, borrado y negocios/actividades ligadas.
#[component]
pub fn ContactDetail(id: String) -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_c = token_of.clone();
    let id_c = id.clone();
    let mut contact = use_resource(move || {
        let token = token_c.clone();
        let id_v = id_c.clone();
        async move {
            match token {
                Some(t) => api::get_contact(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_d = token_of.clone();
    let id_d = id.clone();
    let mut deals = use_resource(move || {
        let token = token_d.clone();
        let id_v = id_d.clone();
        async move {
            match token {
                Some(t) => api::contact_deals(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_a = token_of.clone();
    let id_a = id.clone();
    let mut activities = use_resource(move || {
        let token = token_a.clone();
        let id_v = id_a.clone();
        async move {
            match token {
                Some(t) => api::contact_activities(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Contacto".to_string(),
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

    let n_deals = match &*deals.read() {
        Some(Ok(list)) => list.len().to_string(),
        _ => "…".to_string(),
    };

    rsx! {
        Layout { title: "Contacto".to_string(),
            Link { to: Route::Contacts {}, class: "link", "← Volver a contactos" }
            match &*contact.read() {
                None => rsx! { p { class: "muted", "Cargando contacto…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar el contacto." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| contact.restart(), "Reintentar" }
                    }
                },
                Some(Ok(c)) => {
                    let detail = c.clone();
                    rsx! {
                        ContactDetailView {
                            contact: detail,
                            deals: deals.read().clone(),
                            activities: activities.read().clone(),
                            on_changed: move || {
                                contact.restart();
                                deals.restart();
                                activities.restart();
                            },
                        }
                    }
                },
            }
        }
    }
}

/// Vista del detalle: cabecera, barra lateral y pestañas (como el original).
#[component]
fn ContactDetailView(
    contact: api::Contact,
    deals: Option<Result<Vec<api::Deal>, ApiError>>,
    activities: Option<Result<Vec<api::Activity>, ApiError>>,
    on_changed: EventHandler<()>,
) -> Element {
    let mut tab = use_signal(|| "datos".to_string());
    let mut editing = use_signal(|| false);

    let id = contact.id.to_string();
    let name = contact.full_name();
    let initials = contact_initials(&contact.first_name, &contact.last_name);
    let position = contact.position.clone().unwrap_or_default();
    let email = contact.email.clone().unwrap_or_default();
    let phone = contact.phone.clone().unwrap_or_default();
    let notes = contact.notes.clone().unwrap_or_default();
    let created = contact.created_at.format("%d %b %Y").to_string();
    let n_deals = match &deals {
        Some(Ok(l)) => l.len(),
        _ => 0,
    };
    let n_act = match &activities {
        Some(Ok(l)) => l.len(),
        _ => 0,
    };
    let current = tab.read().clone();
    let editing_open = *editing.read();

    rsx! {
        Link { to: Route::Contacts {}, class: "link", "← Volver a contactos" }
        section { class: "card",
            div { class: "detail-head",
                span { class: "avatar avatar-lg", "{initials}" }
                div {
                    h2 { class: "card-title", "{name}" }
                    if !position.is_empty() {
                        p { class: "muted small", "{position}" }
                    }
                }
                div { class: "row detail-actions",
                    button {
                        class: "btn btn-small",
                        onclick: move |_| editing.set(!editing_open),
                        if editing_open { "Cerrar edición" } else { "Editar" }
                    }
                    DeleteContactInline { id: id.clone(), name: name.clone() }
                }
            }
        }
        div { class: "detail-grid",
            div {
                section { class: "card",
                    h2 { class: "card-title", "Detalles" }
                    dl { class: "detail-list",
                        if !email.is_empty() {
                            div {
                                dt { "Email" }
                                dd {
                                    a { class: "link", href: "mailto:{email}", "{email}" }
                                }
                            }
                        }
                        if !phone.is_empty() {
                            div {
                                dt { "Teléfono" }
                                dd {
                                    a { class: "link", href: "tel:{phone}", "{phone}" }
                                }
                            }
                        }
                        if !position.is_empty() {
                            div {
                                dt { "Cargo" }
                                dd { "{position}" }
                            }
                        }
                        div {
                            dt { "Alta" }
                            dd { "{created}" }
                        }
                    }
                }
                section { class: "card",
                    h2 { class: "card-title", "Resumen" }
                    div { class: "row",
                        div {
                            p { class: "kpi-value", "{n_deals}" }
                            p { class: "muted small", "negocios" }
                        }
                        div {
                            p { class: "kpi-value", "{n_act}" }
                            p { class: "muted small", "actividades" }
                        }
                    }
                }
                EntityTags { entity_type: "contact".to_string(), entity_id: id.clone() }
                if !notes.is_empty() {
                    section { class: "card",
                        h2 { class: "card-title", "Notas" }
                        p { class: "detail-notes", "{notes}" }
                    }
                }
            }
            div {
                div { class: "row tabs",
                    button {
                        class: if current == "datos" { "btn btn-small btn-primary" } else { "btn btn-small" },
                        onclick: move |_| tab.set("datos".to_string()),
                        "Datos"
                    }
                    button {
                        class: if current == "negocios" { "btn btn-small btn-primary" } else { "btn btn-small" },
                        onclick: move |_| tab.set("negocios".to_string()),
                        "Negocios ({n_deals})"
                    }
                    button {
                        class: if current == "actividad" { "btn btn-small btn-primary" } else { "btn btn-small" },
                        onclick: move |_| tab.set("actividad".to_string()),
                        "Actividad ({n_act})"
                    }
                }
                if current == "datos" {
                    if *editing.read() {
                        ContactEditor { contact: contact.clone(), on_saved: move || on_changed.call(()) }
                    } else {
                        p { class: "muted", "Pulsa «Editar» para modificar la ficha." }
                    }
                } else if current == "negocios" {
                    section { class: "card",
                        h2 { class: "card-title", "Negocios" }
                        match deals {
                            None => rsx! { p { class: "muted", "Cargando…" } },
                            Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                            Some(Ok(list)) => rsx! {
                                if list.is_empty() {
                                    p { class: "muted", "Sin negocios ligados." }
                                } else {
                                    div { class: "table-wrap",
                                        table { class: "table",
                                            thead {
                                                tr {
                                                    th { "Nombre" }
                                                    th { "Valor" }
                                                    th { "Etapa" }
                                                }
                                            }
                                            tbody {
                                                for d in &list {
                                                    {
                                                        let did = d.id.to_string();
                                                        let title = d.title.clone();
                                                        let amount = format!("${:.0} {}", d.value, d.currency);
                                                        let stage = api::deal_stage_label(&d.stage).to_string();
                                                        rsx! {
                                                            tr { key: "{did}",
                                                                td {
                                                                    Link {
                                                                        to: Route::DealDetail { id: did },
                                                                        class: "link",
                                                                        "{title}"
                                                                    }
                                                                }
                                                                td { "{amount}" }
                                                                td { "{stage}" }
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
                    }
                } else {
                    section { class: "card",
                        h2 { class: "card-title", "Actividad" }
                        match activities {
                            None => rsx! { p { class: "muted", "Cargando…" } },
                            Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                            Some(Ok(list)) => rsx! {
                                if list.is_empty() {
                                    p { class: "muted", "Sin actividad registrada." }
                                } else {
                                    ul { class: "timeline",
                                        for a in list.iter().take(20) {
                                            {
                                                let key = a.id.to_string();
                                                let subject = a.subject.clone();
                                                let kind = a.activity_type.clone();
                                                let when = a.created_at.format("%d %b %Y").to_string();
                                                let done = a.completed;
                                                rsx! {
                                                    li { key: "{key}", class: "timeline-item",
                                                        span { class: "timeline-dot" }
                                                        div {
                                                            strong { "{subject}" }
                                                            p { class: "muted small",
                                                                "{kind} · {when}"
                                                                if done {
                                                                    " · hecha ✓"
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
                    }
                }
            }
        }
    }
}

/// Borrado inline con confirmación.
#[component]
fn DeleteContactInline(id: String, name: String) -> Element {
    let auth = use_auth();
    let nav = use_navigator();
    let mut confirm = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let asking = *confirm.read();
    let busy_v = *busy.read();

    rsx! {
        if !asking {
            button {
                class: "btn btn-small danger-btn",
                onclick: move |_| confirm.set(true),
                "Eliminar"
            }
        } else {
            span { class: "muted small", "¿Eliminar «{name}»?" }
            button {
                class: "btn btn-small danger-btn",
                disabled: busy_v,
                onclick: move |_| {
                    let token = auth.read().session.clone().map(|s| s.access_token);
                    let id_v = id.clone();
                    busy.set(true);
                    spawn(async move {
                        let res = match token {
                            Some(t) => api::delete_contact(&t, &id_v).await.map_err(|e| e.to_string()),
                            None => Err("Sin sesión.".to_string()),
                        };
                        busy.set(false);
                        match res {
                            Ok(()) => {
                                nav.push(Route::Contacts {});
                            }
                            Err(msg) => {
                                error.set(Some(msg));
                                confirm.set(false);
                            }
                        }
                    });
                },
                if busy_v { "…" } else { "Sí" }
            }
            button {
                class: "btn btn-small",
                onclick: move |_| confirm.set(false),
                "No"
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error small", "{msg}" }
            }
        }
    }
}
#[component]
fn ContactEditor(contact: api::Contact, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut first = use_signal(|| contact.first_name.clone());
    let mut last = use_signal(|| contact.last_name.clone());
    let mut email = use_signal(|| contact.email.clone().unwrap_or_default());
    let mut phone = use_signal(|| contact.phone.clone().unwrap_or_default());
    let mut position = use_signal(|| contact.position.clone().unwrap_or_default());
    let mut notes = use_signal(|| contact.notes.clone().unwrap_or_default());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);

    let name = contact.full_name();
    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Guardar cambios" };
    let was_saved = *saved.read();

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let id_v = contact.id.to_string();
                let f = first.read().trim().to_string();
                let l = last.read().trim().to_string();
                if f.is_empty() || l.is_empty() {
                    error.set(Some("Nombre y apellido son obligatorios.".to_string()));
                    return;
                }
                let em = none_if_empty(email.read().trim());
                let ph = none_if_empty(phone.read().trim());
                let pos = none_if_empty(position.read().trim());
                let no = none_if_empty(notes.read().trim());
                busy.set(true);
                error.set(None);
                saved.set(false);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::update_contact(
                                    &t,
                                    &id_v,
                                    &api::UpdateContactBody {
                                        first_name: Some(&f),
                                        last_name: Some(&l),
                                        email: em.as_deref(),
                                        phone: ph.as_deref(),
                                        company_id: None,
                                        position: pos.as_deref(),
                                        notes: no.as_deref(),
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
                        Ok(()) => {
                            saved.set(true);
                            on_saved.call(());
                        }
                        Err(msg) => error.set(Some(msg)),
                    }
                });
            },
            h2 { class: "card-title", "{name}" }
            div { class: "row",
                div {
                    label { class: "field", "Nombre *" }
                    input {
                        class: "input",
                        value: "{first}",
                        oninput: move |e| first.set(e.value()),
                        required: true,
                    }
                }
                div {
                    label { class: "field", "Apellido *" }
                    input {
                        class: "input",
                        value: "{last}",
                        oninput: move |e| last.set(e.value()),
                        required: true,
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Email" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Teléfono" }
                    input {
                        class: "input",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }
            }
            label { class: "field", "Cargo" }
            input {
                class: "input",
                value: "{position}",
                oninput: move |e| position.set(e.value()),
            }
            label { class: "field", "Notas" }
            textarea {
                class: "input",
                rows: 2,
                value: "{notes}",
                oninput: move |e| notes.set(e.value()),
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
            if was_saved {
                p { class: "alert alert-ok", "Cambios guardados." }
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
