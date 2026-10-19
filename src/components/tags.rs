//! Etiquetas por entidad (`contact|company|deal|lead`): ver, asignar, crear y quitar.

use dioxus::prelude::*;

use crate::{api, auth::use_auth};

/// Etiquetas de una entidad con gestión inline. Requiere sesión.
#[component]
pub fn EntityTags(entity_type: String, entity_id: String) -> Element {
    let auth = use_auth();
    let mut new_name = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut error = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_e = token_of.clone();
    let type_e = entity_type.clone();
    let id_e = entity_id.clone();
    let mut mine = use_resource(move || {
        let token = token_e.clone();
        let et = type_e.clone();
        let eid = id_e.clone();
        async move {
            match token {
                Some(t) => api::entity_tags(&t, &et, &eid).await,
                None => Err(api::ApiError::no_session()),
            }
        }
    });
    let token_a = token_of.clone();
    let mut all = use_resource(move || {
        let token = token_a.clone();
        async move {
            match token {
                Some(t) => api::list_tags(&t).await,
                None => Err(api::ApiError::no_session()),
            }
        }
    });

    let busy_v = *busy.read();
    let token_as = token_of.clone();
    let type_as = entity_type.clone();
    let id_as = entity_id.clone();
    let token_new = token_of.clone();
    let type_new = entity_type.clone();
    let id_new = entity_id.clone();

    rsx! {
        section { class: "card",
            h2 { class: "card-title", "Etiquetas" }
            match &*mine.read() {
                None => rsx! { p { class: "muted", "Cargando…" } },
                Some(Err(e)) => rsx! { p { class: "muted small", "Sin acceso a etiquetas: {e}" } },
                Some(Ok(list)) => rsx! {
                    if list.is_empty() {
                        p { class: "muted", "Sin etiquetas." }
                    } else {
                        div { class: "row",
                            for t in list {
                                {
                                    let key = t.id.to_string();
                                    let name = t.name.clone();
                                    let token_v = token_of.clone();
                                    let et_v = entity_type.clone();
                                    let eid_v = entity_id.clone();
                                    rsx! {
                                        span { key: "{key}", class: "pill",
                                            "{name} "
                                            button {
                                                class: "link-btn small",
                                                disabled: busy_v,
                                                onclick: move |_| {
                                                    let tok = token_v.clone();
                                                    let tid = key.clone();
                                                    let et = et_v.clone();
                                                    let eid = eid_v.clone();
                                                    spawn(async move {
                                                        if let Some(tk) = tok {
                                                            let _ = api::remove_tag(&tk, &tid, &et, &eid).await;
                                                        }
                                                        mine.restart();
                                                    });
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
            }
            if let Some(Ok(tags)) = &*all.read() {
                {
                    let mine_ids: Vec<String> = match &*mine.read() {
                        Some(Ok(list)) => list.iter().map(|t| t.id.to_string()).collect(),
                        _ => Vec::new(),
                    };
                    let free: Vec<api::Tag> =
                        tags.iter().filter(|t| !mine_ids.contains(&t.id.to_string())).cloned().collect();
                    rsx! {
                        div { class: "row",
                            select {
                                class: "input",
                                id: "tag-pick",
                                option { value: "", "Elegir existente…" }
                                for t in free {
                                    {
                                        let tid = t.id.to_string();
                                        let tname = t.name.clone();
                                        rsx! { option { value: "{tid}", "{tname}" } }
                                    }
                                }
                            }
                            button {
                                class: "btn btn-small",
                                disabled: busy_v,
                                onclick: move |_| {
                                    let tok = token_as.clone();
                                    let et_v = type_as.clone();
                                    let eid_v = id_as.clone();
                                    spawn(async move {
                                        let chosen = current_select_value("tag-pick");
                                        if let (Some(tk), Some(tid)) = (tok, chosen) {
                                            busy.set(true);
                                            let res = api::assign_tag(
                                                    &tk,
                                                    &api::AssignTagBody {
                                                        tag_id: &tid,
                                                        entity_type: &et_v,
                                                        entity_id: &eid_v,
                                                    },
                                                )
                                                .await;
                                            busy.set(false);
                                            if res.is_ok() {
                                                mine.restart();
                                            } else {
                                                error.set(Some("No se pudo asignar.".to_string()));
                                            }
                                        }
                                    });
                                },
                                "Asignar"
                            }
                        }
                    }
                }
            }
            div { class: "row",
                input {
                    class: "input",
                    placeholder: "Nueva etiqueta…",
                    value: "{new_name}",
                    oninput: move |e| new_name.set(e.value()),
                }
                button {
                    class: "btn btn-small",
                    disabled: busy_v,
                    onclick: move |_| {
                        let tok = token_new.clone();
                        let et_v = type_new.clone();
                        let eid_v = id_new.clone();
                        let name_v = new_name.read().trim().to_string();
                        if name_v.is_empty() {
                            return;
                        }
                        busy.set(true);
                        error.set(None);
                        spawn(async move {
                            let res = match tok {
                                Some(tk) => {
                                    match api::create_tag(
                                        &tk,
                                        &api::CreateTagBody { name: &name_v, color: None },
                                    )
                                    .await
                                    {
                                        Ok(tag) => {
                                            api::assign_tag(
                                                &tk,
                                                &api::AssignTagBody {
                                                    tag_id: &tag.id.to_string(),
                                                    entity_type: &et_v,
                                                    entity_id: &eid_v,
                                                },
                                            )
                                            .await
                                        }
                                        Err(e) => Err(e),
                                    }
                                }
                                None => Err(api::ApiError::no_session()),
                            };
                            busy.set(false);
                            match res {
                                Ok(()) => {
                                    new_name.set(String::new());
                                    mine.restart();
                                    all.restart();
                                }
                                Err(e) => error.set(Some(e.to_string())),
                            }
                        });
                    },
                    "Crear y asignar"
                }
            }
            if let Some(msg) = error.read().clone() {
                p { class: "alert alert-error", "{msg}" }
            }
        }
    }
}

/// Lee el valor actual de un `<select>` por id (solo navegador).
fn current_select_value(element_id: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        web_sys::window()?
            .document()?
            .get_element_by_id(element_id)?
            .dyn_into::<web_sys::HtmlSelectElement>()
            .ok()
            .map(|s| s.value())
            .filter(|v| !v.is_empty())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element_id;
        None
    }
}
