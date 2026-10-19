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
        async move {
            match token {
                Some(t) => api::list_contacts(&t, p, pp, &q, &sort_v, &order_v).await,
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
                select {
                    class: "input",
                    value: "{sort_by}",
                    onchange: move |e| {
                        sort_by.set(e.value());
                        page.set(1);
                    },
                    for (value, label) in SORT_OPTIONS {
                        option { value, "{label}" }
                    }
                }
                select {
                    class: "input",
                    value: "{order}",
                    onchange: move |e| {
                        order.set(e.value());
                        page.set(1);
                    },
                    option { value: "desc", "Desc" }
                    option { value: "asc", "Asc" }
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
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
                    rsx! {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "" }
                                        th { "Nombre" }
                                        th { "Email" }
                                        th { "Teléfono" }
                                        th { "Cargo" }
                                    }
                                }
                                tbody {
                                    for c in &paged.data {
                                        {
                                            let id = c.id.to_string();
                                            let is_sel = selected.read().contains(&id);
                                            let id_t = id.clone();
                                            let name = c.full_name();
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
                    let detail_id = c.id.to_string();
                    let detail_name = c.full_name();
                    rsx! {
                        ContactEditor { contact: c.clone(), on_saved: move || contact.restart() }
                        EntityTags { entity_type: "contact".to_string(), entity_id: detail_id.clone() }
                        DeleteContact { id: detail_id, name: detail_name }
                        section { class: "card",
                            h2 { class: "card-title", "Negocios ({n_deals})" }
                            match &*deals.read() {
                                None => rsx! { p { class: "muted", "Cargando…" } },
                                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                                Some(Ok(list)) => rsx! {
                                    if list.is_empty() {
                                        p { class: "muted", "Sin negocios ligados." }
                                    } else {
                                        ul {
                                            for d in list {
                                                {
                                                    let did = d.id.to_string();
                                                    let title = d.title.clone();
                                                    let stage = api::deal_stage_label(&d.stage).to_string();
                                                    let amount =
                                                        format!("{:.0} {}", d.value, d.currency);
                                                    rsx! {
                                                        li { key: "{did}",
                                                            Link {
                                                                to: Route::DealDetail { id: did },
                                                                class: "link",
                                                                "{title}"
                                                            }
                                                            span { class: "muted small",
                                                                " · {stage} · {amount}"
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
                        section { class: "card",
                            h2 { class: "card-title", "Actividad reciente" }
                            match &*activities.read() {
                                None => rsx! { p { class: "muted", "Cargando…" } },
                                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudo cargar: {e}" } },
                                Some(Ok(list)) => rsx! {
                                    if list.is_empty() {
                                        p { class: "muted", "Sin actividad registrada." }
                                    } else {
                                        ul {
                                            for a in list.iter().take(10) {
                                                {
                                                    let key = a.id.to_string();
                                                    let subject = a.subject.clone();
                                                    let kind = a.activity_type.clone();
                                                    let when = a.created_at.format("%Y-%m-%d").to_string();
                                                    rsx! {
                                                        li { key: "{key}",
                                                            strong { "{subject}" }
                                                            span { class: "muted small",
                                                                " · {kind} · {when}"
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
                },
            }
        }
    }
}

/// Formulario de edición precargado.
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

/// Borrado con confirmación en dos pasos.
#[component]
fn DeleteContact(id: String, name: String) -> Element {
    let auth = use_auth();
    let nav = use_navigator();
    let mut confirm = use_signal(|| false);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let asking = *confirm.read();
    let busy_v = *busy.read();
    let yes_label = if busy_v { "Eliminando…" } else { "Sí, eliminar" };

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Zona de peligro" }
            if !asking {
                button {
                    class: "btn btn-small",
                    onclick: move |_| confirm.set(true),
                    "Eliminar contacto"
                }
            } else {
                p { "¿Eliminar «{name}» definitivamente?" }
                if let Some(msg) = error.read().clone() {
                    p { class: "alert alert-error", "{msg}" }
                }
                div { class: "row",
                    button {
                        class: "btn btn-small",
                        disabled: busy_v,
                        onclick: move |_| {
                            let token = auth
                                .read()
                                .session
                                .clone()
                                .map(|s| s.access_token);
                            let id_v = id.clone();
                            busy.set(true);
                            spawn(async move {
                                let res = match token {
                                    Some(t) => {
                                        api::delete_contact(&t, &id_v)
                                            .await
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(()) => {
                                        nav.push(Route::Contacts {});
                                    }
                                    Err(msg) => error.set(Some(msg)),
                                }
                            });
                        },
                        "{yes_label}"
                    }
                    button {
                        class: "btn btn-small",
                        onclick: move |_| {
                            confirm.set(false);
                            error.set(None);
                        },
                        "Cancelar"
                    }
                }
            }
        }
    }
}
