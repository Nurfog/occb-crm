//! 404 de la SPA.

use dioxus::prelude::*;

use crate::{components::Layout, route::Route};

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));
    rsx! {
        Layout { title: "No encontrado".to_string(),
            p {
                "La ruta "
                code { "{path}" }
                " no existe en este frontend."
            }
            Link { to: Route::Home {}, class: "btn btn-small btn-primary", "Volver al inicio" }
        }
    }
}
