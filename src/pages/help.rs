//! Ayuda: qué hay en cada sección y cómo pedir tus derechos.

use dioxus::prelude::*;

use crate::{components::Layout, route::Route};

/// Página de ayuda estática.
#[component]
pub fn Help() -> Element {
    rsx! {
        Layout { title: "Ayuda".to_string(),
            section { class: "card",
                h2 { class: "card-title", "El día a día" }
                ul {
                    li {
                        strong { "Prospectos" }
                        " Captura interesados, anótales la bitácora y conviértelos cuando estén listos: crea contacto, empresa y negocio de una vez."
                    }
                    li {
                        strong { "Contactos y Empresas" }
                        " Fichas con edición, etiquetas, negocios ligados e ingresos por empresa."
                    }
                    li {
                        strong { "Negocios" }
                        " Tablero por etapas: arrastra con el selector de cada tarjeta (o edita en el detalle). Importa y exporta por CSV."
                    }
                    li {
                        strong { "Actividades y Calendario" }
                        " Tareas y reuniones: márcalas hechas al terminar."
                    }
                    li {
                        strong { "Informes" }
                        " Pipeline por etapa y tasa de ganados/perdidos."
                    }
                }
            }
            section { class: "card",
                h2 { class: "card-title", "Comunicación" }
                ul {
                    li {
                        strong { "Email" }
                        " Redacta con o sin plantilla y revisa la bitácora (requiere SMTP activo)."
                    }
                    li {
                        strong { "WhatsApp" }
                        " Configura el número y la cuenta, conversa y envía (requiere config activa)."
                    }
                    li {
                        strong { "Notificaciones" }
                        " La campana del menú muestra las no leídas; ajusta tus preferencias ahí mismo."
                    }
                }
            }
            section { class: "card",
                h2 { class: "card-title", "Cuenta y privacidad" }
                ul {
                    li {
                        "Cambia tu clave en "
                        Link { to: Route::Dashboard {}, class: "link", "Panel → Seguridad" }
                        ". Si la olvidaste: "
                        Link { to: Route::ForgotPassword {}, class: "link", "recupérala aquí" }
                        "."
                    }
                    li {
                        "Tus derechos sobre tus datos (acceso, rectificación, supresión, oposición, portabilidad, bloqueo): "
                        Link { to: Route::Rights {}, class: "link", "ejércelos aquí" }
                        "."
                    }
                    li {
                        Link { to: Route::Privacy {}, class: "link", "Política de Privacidad" }
                        " — cómo tratamos tus datos según la Ley 21.719."
                    }
                }
            }
        }
    }
}
