//! Portada: presentación del sistema (landing) + estado del backend.
//!
//! La privacidad (Ley 21.719) vive en el pie de página, no aquí.

use dioxus::prelude::*;

use crate::{api, components::Layout, route::Route};

const HERO_IMG: &str = "https://images.unsplash.com/photo-1497366216548-37526070297c?auto=format&fit=crop&w=1600&q=70";
const TEAM_IMG: &str = "https://images.unsplash.com/photo-1522071820081-009f0129c71c?auto=format&fit=crop&w=900&q=70";
const DATA_IMG: &str = "https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=900&q=70";
const MEET_IMG: &str = "https://images.unsplash.com/photo-1552664730-d307ca884978?auto=format&fit=crop&w=900&q=70";

/// Landing de presentación. Sin sesión muestra el producto; con sesión,
// el panel sigue en `/dashboard`.
#[component]
pub fn Home() -> Element {
    let health = use_resource(|| async move { api::health().await });

    let status_pill = match &*health.read() {
        Some(Ok(h)) => {
            let ver = h.version.clone().unwrap_or_default();
            let tag = if ver.is_empty() { "Sistema en línea".to_string() } else { format!("Sistema en línea · v{ver}") };
            rsx! {
                p { class: "hero-status",
                    span { class: "dot dot-ok" }
                    "{tag}"
                }
            }
        }
        _ => rsx! {},
    };

    rsx! {
        Layout { title: "".to_string(),
            section { class: "hero", style: "background-image: url({HERO_IMG})",
                div { class: "hero-overlay",
                    div { class: "hero-inner",
                        p { class: "hero-eyebrow", "OCCB · CRM" }
                        h1 { class: "hero-title",
                            "Toda tu relación con clientes, en un solo lugar"
                        }
                        p { class: "hero-sub",
                            "Prospectos, contactos, empresas y negocios con tablero kanban, "
                            "actividades, documentos, informes y comunicación integrada. "
                            "Tus datos en tu infraestructura, bajo Ley 21.719."
                        }
                        div { class: "row hero-cta",
                            Link {
                                to: Route::Register {},
                                class: "btn btn-primary btn-large",
                                "Crear cuenta gratis"
                            }
                            Link {
                                to: Route::Login {},
                                class: "btn btn-large hero-btn-ghost",
                                "Entrar"
                            }
                        }
                        {status_pill}
                    }
                }
            }

            section { class: "stats-strip",
                div { class: "stat",
                    strong { "8+" }
                    span { "módulos integrados" }
                }
                div { class: "stat",
                    strong { "100%" }
                    span { "datos bajo tu control" }
                }
                div { class: "stat",
                    strong { "6" }
                    span { "etapas de venta" }
                }
                div { class: "stat",
                    strong { "Ley 21.719" }
                    span { "privacidad incluida" }
                }
            }

            section {
                h2 { class: "section-title", "Un sistema, todo el ciclo comercial" }
                p { class: "subtitle",
                    "Desde el primer contacto hasta el cierre y la postventa."
                }
                div { class: "grid-cards",
                    div { class: "card",
                        h3 { class: "card-title", "Prospectos" }
                        p { class: "muted",
                            "Captura interesados, califícalos por puntaje y conviértelos en contacto, empresa y negocio con un clic."
                        }
                        Link { to: Route::Leads {}, class: "link", "Ver prospectos →" }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "Contactos y empresas" }
                        p { class: "muted",
                            "Fichas completas con etiquetas, actividad ligada e ingresos por empresa."
                        }
                        Link { to: Route::Contacts {}, class: "link", "Ver contactos →" }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "Negocios" }
                        p { class: "muted",
                            "Tablero kanban de 6 etapas con valores, cierres esperados e importación por CSV."
                        }
                        Link { to: Route::Deals {}, class: "link", "Ver tablero →" }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "Actividades y calendario" }
                        p { class: "muted",
                            "Tareas, llamadas y reuniones con vencimientos, conexión a Google y sincronización."
                        }
                        Link { to: Route::Activities {}, class: "link", "Ver actividades →" }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "Documentos" }
                        p { class: "muted",
                            "Sube, descarga y organiza los archivos de cada negocio y cliente."
                        }
                        Link { to: Route::Documents {}, class: "link", "Ver documentos →" }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "Informes y más" }
                        p { class: "muted",
                            "Pipeline por etapa, ganados vs perdidos, email, WhatsApp, auditoría y administración."
                        }
                        Link { to: Route::Reports {}, class: "link", "Ver informes →" }
                    }
                }
            }

            section {
                h2 { class: "section-title", "Hecho para equipos que venden" }
                div { class: "feature-row",
                    img { class: "feature-img", src: "{TEAM_IMG}", alt: "Equipo trabajando junto" }
                    div {
                        h3 { "Trabajo en equipo" }
                        p { class: "muted",
                            "Reparto automático de prospectos, notificaciones, bitácora de actividad "
                            "y permisos por perfil: cada persona ve lo que le toca."
                        }
                    }
                }
                div { class: "feature-row feature-reverse",
                    img { class: "feature-img", src: "{DATA_IMG}", alt: "Análisis de datos en un computador" }
                    div {
                        h3 { "Decisiones con datos" }
                        p { class: "muted",
                            "Estadísticas del negocio, mejores negocios, conversión de prospectos y "
                            "reportes exportables. Sin planillas paralelas."
                        }
                    }
                }
                div { class: "feature-row",
                    img { class: "feature-img", src: "{MEET_IMG}", alt: "Reunión de planificación" }
                    div {
                        h3 { "Comunicación integrada" }
                        p { class: "muted",
                            "Email con plantillas y bitácora, WhatsApp con hilos por contacto y "
                            "calendario sincronizado con Google."
                        }
                    }
                }
            }

            section {
                h2 { class: "section-title", "Empieza en 3 pasos" }
                div { class: "grid-cards",
                    div { class: "card",
                        h3 { class: "card-title", "1. Crea tu cuenta" }
                        p { class: "muted",
                            "Regístrate en un minuto. La primera cuenta queda como administradora."
                        }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "2. Carga tus datos" }
                        p { class: "muted",
                            "Importa tus contactos y empresas por CSV o créalos a mano."
                        }
                    }
                    div { class: "card",
                        h3 { class: "card-title", "3. Vende y mide" }
                        p { class: "muted",
                            "Mueve negocios por el tablero y sigue el avance en informes."
                        }
                    }
                }
            }

            section { class: "cta-band",
                h2 { "¿Listo para ordenar tus ventas?" }
                p { "Crea tu cuenta y ten tu primer negocio en el tablero hoy mismo." }
                div { class: "row",
                    Link {
                        to: Route::Register {},
                        class: "btn btn-primary btn-large",
                        "Crear cuenta"
                    }
                    Link { to: Route::Login {}, class: "btn btn-large hero-btn-ghost", "Entrar" }
                }
            }

            section { class: "card privacy-strip",
                h2 { class: "card-title", "Tus datos, protegidos" }
                p { class: "muted",
                    "Operamos bajo la Ley 21.719 de protección de datos: consentimiento opt-in, "
                    "canal gratuito de derechos y aviso de brechas. Detalles al pie de página."
                }
                div { class: "row",
                    Link { to: Route::Privacy {}, class: "link", "Política de Privacidad →" }
                    Link { to: Route::Rights {}, class: "link", "Ejercer derechos →" }
                }
            }
        }
    }
}
