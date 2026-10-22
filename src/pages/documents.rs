//! Documentos: listado, descarga y borrado (`/api/v1/documents`).
//!
//! La subida queda fuera del MVP web (multipart): se gestiona vía API directa.

use dioxus::prelude::*;

use crate::{
    api::{self, ApiError},
    auth::use_auth,
    components::Layout,
    route::Route,
};

/// Descarga bytes como archivo. Solo en el navegador.
fn download_bytes(filename: &str, bytes: &[u8], mime: &str) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        use web_sys::{Blob, BlobPropertyBag, Url};

        let window = web_sys::window().ok_or("Sin ventana de navegador")?;
        let document = window.document().ok_or("Sin documento")?;
        let bytes_js = js_sys::Uint8Array::from(bytes);
        let parts = js_sys::Array::of1(&bytes_js.into());
        let options = BlobPropertyBag::new();
        options.set_type(mime);
        let blob = Blob::new_with_u8_array_sequence_and_options(&parts, &options)
            .map_err(|_| "No se pudo crear el archivo")?;
        let url = Url::create_object_url_with_blob(&blob)
            .map_err(|_| "No se pudo crear la URL de descarga")?;
        let anchor: web_sys::HtmlAnchorElement = document
            .create_element("a")
            .map_err(|_| "No se pudo crear el enlace")?
            .dyn_into()
            .map_err(|_| "Enlace inválido")?;
        anchor.set_href(&url);
        anchor.set_download(filename);
        anchor.click();
        let _ = Url::revoke_object_url(&url);
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (filename, bytes, mime);
        Err("Descarga disponible solo en el navegador".to_string())
    }
}

/// Listado con búsqueda + descarga + borrado. Requiere sesión.
#[component]
pub fn Documents() -> Element {
    let auth = use_auth();
    let mut search = use_signal(String::new);
    let mut applied = use_signal(String::new);
    let mut folder = use_signal(String::new);
    let mut busy_id = use_signal(|| None::<String>);

    let token_of = auth.read().session.clone().map(|s| s.access_token);
    let token_up = token_of.clone();
    let mut up_folder = use_signal(String::new);
    let mut up_msg = use_signal(|| None::<String>);
    let mut up_busy = use_signal(|| false);
    let mut items = use_resource(move || {
        let token = token_of.clone();
        let q = applied.read().clone();
        let f = folder.read().clone();
        async move {
            match token {
                Some(t) => api::list_documents(&t, &q, &f).await,
                None => Err(ApiError::no_session()),
            }
        }
    });

    if auth.read().session.is_none() {
        return rsx! {
            Layout { title: "Documentos".to_string(),
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
        Layout { title: "Documentos".to_string(),
            div { class: "card form",
                h2 { class: "card-title", "Subir archivo" }
                div { class: "row",
                    input {
                        r#type: "file",
                        id: "doc-upload",
                    }
                    select {
                        class: "input",
                        value: "{up_folder}",
                        onchange: move |e| up_folder.set(e.value()),
                        option { value: "", "Sin carpeta" }
                        for f in DOC_FOLDERS {
                            option { value: f, "{f}" }
                        }
                    }
                    button {
                        class: "btn btn-small btn-primary",
                        disabled: *up_busy.read(),
                        onclick: move |_| {
                            let tok = token_up.clone();
                            let folder_v = up_folder.read().trim().to_string();
                            up_msg.set(None);
                            up_busy.set(true);
                            spawn(async move {
                                let outcome = match crate::files::read_first_file("doc-upload").await {
                                    Ok((name, mime, bytes)) => match tok {
                                        Some(t) => {
                                            let folder_opt = if folder_v.is_empty() {
                                                None
                                            } else {
                                                Some(folder_v.as_str())
                                            };
                                            let mime_v = if mime.is_empty() {
                                                "application/octet-stream"
                                            } else {
                                                mime.as_str()
                                            };
                                            match api::upload_document(&t, &name, mime_v, folder_opt, &bytes)
                                                .await
                                            {
                                                Ok(up) => {
                                                    items.restart();
                                                    format!("Subido: {}.", up.original_name)
                                                }
                                                Err(e) => e.to_string(),
                                            }
                                        }
                                        None => "Sin sesión.".to_string(),
                                    },
                                    Err(e) => e,
                                };
                                up_msg.set(Some(outcome));
                                up_busy.set(false);
                            });
                        },
                        "Subir"
                    }
                }
                if let Some(msg) = up_msg.read().clone() {
                    p { class: "alert alert-info", "{msg}" }
                }
            }
            form {
                class: "toolbar card",
                onsubmit: move |e| {
                    e.prevent_default();
                    applied.set(search.read().clone());
                },
                input {
                    class: "input",
                    r#type: "search",
                    placeholder: "Buscar por nombre…",
                    value: "{search}",
                    oninput: move |e| search.set(e.value()),
                }
                button { class: "btn btn-small btn-primary", r#type: "submit", "Buscar" }
            }
            div { class: "row",
                button {
                    class: if folder.read().is_empty() { "btn btn-small btn-primary" } else { "btn btn-small" },
                    onclick: move |_| folder.set(String::new()),
                    "Todas"
                }
                for f in DOC_FOLDERS {
                    {
                        let active = folder.read().clone() == f;
                        let cls = if active { "btn btn-small btn-primary" } else { "btn btn-small" };
                        let fv = f.to_string();
                        let label = f.to_string();
                        rsx! {
                            button { key: "{f}", class: "{cls}",
                                onclick: move |_| folder.set(fv.clone()),
                                "{label}"
                            }
                        }
                    }
                }
            }
            match &*items.read() {
                None => rsx! { p { class: "muted", "Cargando documentos…" } },
                Some(Err(e)) => rsx! {
                    div { class: "alert alert-error",
                        p {
                            if e.status == 403 {
                                "No tienes permiso para ver documentos."
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
                        p { class: "muted", "Sin documentos." }
                    } else {
                        div { class: "table-wrap card",
                            table { class: "table",
                                thead {
                                    tr {
                                        th { "Nombre" }
                                        th { "Carpeta" }
                                        th { "Tamaño" }
                                        th { "Acciones" }
                                    }
                                }
                                tbody {
                                    for d in list {
                                        {
                                            let key = d.id.to_string();
                                            let name = d.original_name.clone();
                                            let ext = name
                                                .rsplit('.')
                                                .next()
                                                .unwrap_or("")
                                                .to_ascii_uppercase();
                                            let folder_name =
                                                d.folder.clone().unwrap_or_else(|| "—".to_string());
                                            let size = format_size(d.file_size);
                                            let downloading = busy_id.read().clone() == Some(key.clone());
                                            let token_base = auth
                                                .read()
                                                .session
                                                .clone()
                                                .map(|s| s.access_token);
                                            let token_dl = token_base.clone();
                                            let token_del = token_base.clone();
                                            let key_dl = key.clone();
                                            let key_del = key.clone();
                                            rsx! {
                                                tr { key: "{key}",
                                                    td {
                                                        span { class: "tag tag-muted", "{ext}" }
                                                        " {name}"
                                                    }
                                                    td {
                                                        span { class: "tag", "{folder_name}" }
                                                    }
                                                    td { class: "muted", "{size}" }
                                                    td {
                                                        button {
                                                            class: "btn btn-small",
                                                            disabled: downloading,
                                                            onclick: move |_| {
                                                                let token_v = token_dl.clone();
                                                                let id_v = key_dl.clone();
                                                                let name_v = name.clone();
                                                                busy_id.set(Some(id_v.clone()));
                                                                spawn(async move {
                                                                    if let Some(t) = token_v {
                                                                        if let Ok(meta) = api::get_document(&t, &id_v).await {
                                                                            if let Ok(bytes) = api::download_document(&t, &id_v)
                                                                                .await
                                                                            {
                                                                                let mime_v = meta
                                                                                    .mime_type
                                                                                    .unwrap_or_else(|| {
                                                                                        "application/octet-stream".to_string()
                                                                                    });
                                                                                let _ = download_bytes(
                                                                                    &meta.original_name,
                                                                                    &bytes,
                                                                                    &mime_v,
                                                                                );
                                                                            }
                                                                        }
                                                                    }
                                                                    busy_id.set(None);
                                                                });
                                                            },
                                                            if downloading { "…" } else { "Descargar" }
                                                        }
                                                        button {
                                                            class: "btn btn-small",
                                                            onclick: move |_| {
                                                                let token_v = token_del.clone();
                                                                let id_v = key_del.clone();
                                                                spawn(async move {
                                                                    if let Some(t) = token_v {
                                                                        let _ = api::delete_document(&t, &id_v).await;
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
        }
    }
}

/// Carpetas conocidas (como el original).
pub const DOC_FOLDERS: [&str; 5] = ["Contracts", "Reports", "Invoices", "Proposals", "Other"];

fn format_size(bytes: i64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
