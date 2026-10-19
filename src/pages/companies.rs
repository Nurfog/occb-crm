//! Empresas: listado, detalle, alta, edición y borrado (`/api/v1/companies`).

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

const SORT_OPTIONS: [(&str, &str); 5] = [
    ("created_at", "Recientes"),
    ("name", "Nombre"),
    ("industry", "Industria"),
    ("city", "Ciudad"),
    ("country", "País"),
];

fn none_if_empty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

/// Listado con búsqueda, orden y paginación + alta. Requiere sesión.
#[component]
pub fn Companies() -> Element {
    let auth = use_auth();
    let mut search = use_signal(String::new);
    let mut applied = use_signal(String::new);
    let mut page = use_signal(|| 1u32);
    let per_page = use_signal(|| 20u32);
    let mut sort_by = use_signal(|| "created_at".to_string());
    let mut order = use_signal(|| "desc".to_string());
    let mut show_new = use_signal(|| false);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let mut items = use_resource(move || {
        let token = token_of.clone();
        let q = applied.read().clone();
        let p = *page.read();
        let pp = *per_page.read();
        let sort_v = sort_by.read().clone();
        let order_v = order.read().clone();
        async move {
            match token {
                Some(t) => api::list_companies(&t, p, pp, &q, &sort_v, &order_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Empresas".to_string(),
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
    let new_label = if creating { "Cerrar formulario" } else { "+ Nueva empresa" };

    rsx! {
        Layout { title: "Empresas".to_string(),
            div { class: "row",
                button {
                    class: "btn btn-small btn-primary",
                    onclick: move |_| show_new.set(!creating),
                    "{new_label}"
                }
            }
            if creating {
                NewCompanyForm { on_created: move || {
                    show_new.set(false);
                    items.restart();
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
                    placeholder: "Buscar por nombre, industria o ciudad…",
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

            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando empresas…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver empresas."
                            } else {
                                "No se pudo cargar el listado."
                            }
                        }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| items.restart(), "Reintentar" }
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
                                        th { "Nombre" }
                                        th { "Industria" }
                                        th { "Ciudad" }
                                        th { "Email" }
                                    }
                                }
                                tbody {
                                    for c in &paged.data {
                                        {
                                            let id = c.id.to_string();
                                            let name = c.name.clone();
                                            let industry = c.industry.clone().unwrap_or_default();
                                            let city = c.city.clone().unwrap_or_default();
                                            let email = c.email.clone().unwrap_or_default();
                                            rsx! {
                                                tr { key: "{id}",
                                                    td {
                                                        Link {
                                                            to: Route::CompanyDetail { id: id.clone() },
                                                            class: "link",
                                                            "{name}"
                                                        }
                                                    }
                                                    td { "{industry}" }
                                                    td { "{city}" }
                                                    td { "{email}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            if paged.data.is_empty() {
                                p { class: "muted", "Sin empresas. Crea la primera con «+ Nueva empresa»." }
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

/// Alta de empresa.
#[component]
fn NewCompanyForm(on_created: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut name = use_signal(String::new);
    let mut industry = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut city = use_signal(String::new);
    let mut country = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Crear empresa" };

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let n = name.read().trim().to_string();
                if n.is_empty() {
                    error.set(Some("El nombre es obligatorio.".to_string()));
                    return;
                }
                let ind = none_if_empty(industry.read().trim());
                let em = none_if_empty(email.read().trim());
                let ph = none_if_empty(phone.read().trim());
                let ci = none_if_empty(city.read().trim());
                let co = none_if_empty(country.read().trim());
                busy.set(true);
                error.set(None);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::create_company(
                                    &t,
                                    &api::CreateCompanyBody {
                                        name: &n,
                                        industry: ind.as_deref(),
                                        website: None,
                                        phone: ph.as_deref(),
                                        email: em.as_deref(),
                                        address: None,
                                        city: ci.as_deref(),
                                        country: co.as_deref(),
                                        notes: None,
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
            h2 { class: "card-title", "Nueva empresa" }
            label { class: "field", "Nombre *" }
            input {
                class: "input",
                value: "{name}",
                oninput: move |e| name.set(e.value()),
                required: true,
            }
            div { class: "row",
                div {
                    label { class: "field", "Industria" }
                    input {
                        class: "input",
                        value: "{industry}",
                        oninput: move |e| industry.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Email" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Teléfono" }
                    input {
                        class: "input",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Ciudad" }
                    input {
                        class: "input",
                        value: "{city}",
                        oninput: move |e| city.set(e.value()),
                    }
                }
            }
            label { class: "field", "País" }
            input {
                class: "input",
                value: "{country}",
                oninput: move |e| country.set(e.value()),
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

/// Detalle con edición, borrado, ingresos y relaciones.
#[component]
pub fn CompanyDetail(id: String) -> Element {
    let auth = use_auth();
    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_c = token_of.clone();
    let id_c = id.clone();
    let mut company = use_resource(move || {
        let token = token_c.clone();
        let id_v = id_c.clone();
        async move {
            match token {
                Some(t) => api::get_company(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_k = token_of.clone();
    let id_k = id.clone();
    let mut contacts = use_resource(move || {
        let token = token_k.clone();
        let id_v = id_k.clone();
        async move {
            match token {
                Some(t) => api::company_contacts(&t, &id_v).await,
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
                Some(t) => api::company_deals(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });
    let token_r = token_of.clone();
    let id_r = id.clone();
    let mut revenue = use_resource(move || {
        let token = token_r.clone();
        let id_v = id_r.clone();
        async move {
            match token {
                Some(t) => api::company_revenue(&t, &id_v).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Empresa".to_string(),
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

    let revenue_line = match &*revenue.read() {
        Some(Ok(r)) => format!("{:.0} en {} negocios", r.total_value, r.deal_count),
        _ => "…".to_string(),
    };

    rsx! {
        Layout { title: "Empresa".to_string(),
            Link { to: Route::Companies {}, class: "link", "← Volver a empresas" }
            match &*company.read() {
                None => rsx! { p { class: "muted", "Cargando empresa…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p { "No se pudo cargar la empresa." }
                        p { class: "muted small", "{e}" }
                        button { class: "btn btn-small", onclick: move |_| company.restart(), "Reintentar" }
                    }
                },
                Some(Ok(c)) => {
                    let detail_id = c.id.to_string();
                    let detail_name = c.name.clone();
                    rsx! {
                        CompanyEditor { company: c.clone(), on_saved: move || company.restart() }
                        DeleteCompany { id: detail_id, name: detail_name }
                        section { class: "card",
                            h2 { class: "card-title", "Ingresos (ganados + en curso)" }
                            p { "{revenue_line}" }
                        }
                        section { class: "card",
                            h2 { class: "card-title", "Contactos" }
                            match &*contacts.read() {
                                None => rsx! { p { class: "muted", "Cargando…" } },
                                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                                Some(Ok(list)) => rsx! {
                                    if list.is_empty() {
                                        p { class: "muted", "Sin contactos." }
                                    } else {
                                        ul {
                                            for k in list {
                                                {
                                                    let kid = k.id.to_string();
                                                    let kname = k.full_name();
                                                    rsx! {
                                                        li { key: "{kid}",
                                                            Link {
                                                                to: Route::ContactDetail { id: kid },
                                                                class: "link",
                                                                "{kname}"
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
                            h2 { class: "card-title", "Negocios" }
                            match &*deals.read() {
                                None => rsx! { p { class: "muted", "Cargando…" } },
                                Some(Err(e)) => rsx! { p { class: "muted small", "No se pudieron cargar: {e}" } },
                                Some(Ok(list)) => rsx! {
                                    if list.is_empty() {
                                        p { class: "muted", "Sin negocios." }
                                    } else {
                                        ul {
                                            for d in list {
                                                {
                                                    let did = d.id.to_string();
                                                    let title = d.title.clone();
                                                    let stage = api::deal_stage_label(&d.stage).to_string();
                                                    let amount = format!("{:.0} {}", d.value, d.currency);
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
                    }
                },
            }
        }
    }
}

/// Edición de la empresa.
#[component]
fn CompanyEditor(company: api::Company, on_saved: EventHandler<()>) -> Element {
    let auth = use_auth();
    let mut name = use_signal(|| company.name.clone());
    let mut industry = use_signal(|| company.industry.clone().unwrap_or_default());
    let mut email = use_signal(|| company.email.clone().unwrap_or_default());
    let mut phone = use_signal(|| company.phone.clone().unwrap_or_default());
    let mut city = use_signal(|| company.city.clone().unwrap_or_default());
    let mut country = use_signal(|| company.country.clone().unwrap_or_default());
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);
    let mut saved = use_signal(|| false);

    let title = company.name.clone();
    let busy_v = *busy.read();
    let submit_label = if busy_v { "Guardando…" } else { "Guardar cambios" };
    let was_saved = *saved.read();

    rsx! {
        form {
            class: "card form",
            onsubmit: move |e| {
                e.prevent_default();
                let token = auth.read().session.clone().map(|s| s.access_token);
                let id_v = company.id.to_string();
                let n = name.read().trim().to_string();
                if n.is_empty() {
                    error.set(Some("El nombre es obligatorio.".to_string()));
                    return;
                }
                let ind = none_if_empty(industry.read().trim());
                let em = none_if_empty(email.read().trim());
                let ph = none_if_empty(phone.read().trim());
                let ci = none_if_empty(city.read().trim());
                let co = none_if_empty(country.read().trim());
                busy.set(true);
                error.set(None);
                saved.set(false);
                spawn(async move {
                    let res = match token {
                        Some(t) => {
                            api::update_company(
                                    &t,
                                    &id_v,
                                    &api::UpdateCompanyBody {
                                        name: Some(&n),
                                        industry: ind.as_deref(),
                                        website: None,
                                        phone: ph.as_deref(),
                                        email: em.as_deref(),
                                        address: None,
                                        city: ci.as_deref(),
                                        country: co.as_deref(),
                                        notes: None,
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
            h2 { class: "card-title", "{title}" }
            label { class: "field", "Nombre *" }
            input {
                class: "input",
                value: "{name}",
                oninput: move |e| name.set(e.value()),
                required: true,
            }
            div { class: "row",
                div {
                    label { class: "field", "Industria" }
                    input {
                        class: "input",
                        value: "{industry}",
                        oninput: move |e| industry.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Email" }
                    input {
                        class: "input",
                        r#type: "email",
                        value: "{email}",
                        oninput: move |e| email.set(e.value()),
                    }
                }
            }
            div { class: "row",
                div {
                    label { class: "field", "Teléfono" }
                    input {
                        class: "input",
                        value: "{phone}",
                        oninput: move |e| phone.set(e.value()),
                    }
                }
                div {
                    label { class: "field", "Ciudad" }
                    input {
                        class: "input",
                        value: "{city}",
                        oninput: move |e| city.set(e.value()),
                    }
                }
            }
            label { class: "field", "País" }
            input {
                class: "input",
                value: "{country}",
                oninput: move |e| country.set(e.value()),
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
fn DeleteCompany(id: String, name: String) -> Element {
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
                    "Eliminar empresa"
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
                                        api::delete_company(&t, &id_v)
                                            .await
                                            .map_err(|e| e.to_string())
                                    }
                                    None => Err("Sin sesión.".to_string()),
                                };
                                busy.set(false);
                                match res {
                                    Ok(()) => {
                                        nav.push(Route::Companies {});
                                    }
                                    Err(msg) => error.set(Some(msg)),
                                }
                            });
                        },
                        "{yes_label}"
                    }
                    button {
                        class: "btn btn-small",
                        onclick: move |_: MouseEvent | {
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
