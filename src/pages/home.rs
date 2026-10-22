//! Portada: presentación del sistema (landing) + estado del backend.
//!
//! La privacidad (Ley 21.719) vive en el pie de página, no aquí.

use dioxus::prelude::*;

use crate::{api, components::Layout, route::Route};

const HERO_IMG: &str = "https://images.unsplash.com/photo-1522071820081-009f0129c71c?auto=format&fit=crop&w=900&q=70";
const TEAM_IMG: &str = "https://images.unsplash.com/photo-1557804506-669a67965ba0?auto=format&fit=crop&w=900&q=70";
const DATA_IMG: &str = "https://images.unsplash.com/photo-1460925895917-afdab827c52f?auto=format&fit=crop&w=900&q=70";
const MEET_IMG: &str = "https://images.unsplash.com/photo-1552664730-d307ca884978?auto=format&fit=crop&w=900&q=70";
const OFFICE_IMG: &str = "https://images.unsplash.com/photo-1497366216548-37526070297c?auto=format&fit=crop&w=900&q=70";
const DEAL_IMG: &str = "https://images.unsplash.com/photo-1600880292203-757bb62b4baf?auto=format&fit=crop&w=900&q=70";

/// Landing de presentación. Sin sesión muestra el producto; con sesión,
// el panel sigue en `/dashboard`.
#[component]
pub fn Home() -> Element {
    let health = use_resource(|| async move { api::health().await });

    let status_pill = match &*health.read() {
        Some(Ok(h)) => {
            let ver = h.version.clone().unwrap_or_default();
            let tag = if ver.is_empty() {
                "Sistema en línea".to_string()
            } else {
                format!("Sistema en línea · v{ver}")
            };
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
            section { class: "hero-pro",
                div { class: "hero-grid",
                    div { class: "hero-copy",
                        p { class: "eyebrow", "OCCB · CRM" }
                        h1 { class: "hero-title",
                            "Toda tu relación con clientes, "
                            span { class: "grad-text", "en un solo lugar" }
                        }
                        p { class: "hero-sub",
                            "Prospectos, contactos, empresas y negocios con tablero kanban, "
                            "actividades, documentos, informes y comunicación integrada. "
                            "Tus datos en tu infraestructura, bajo Ley 21.719."
                        }
                        div { class: "row hero-cta",
                            Link {
                                to: Route::Register {},
                                class: "btn btn-primary btn-large btn-shadow",
                                "Crear cuenta gratis"
                            }
                            Link {
                                to: Route::Login {},
                                class: "btn btn-large btn-outline",
                                "Entrar"
                            }
                        }
                        {status_pill}
                        div { class: "hero-points",
                            span { "✓ Sin tarjetas" }
                            span { "✓ Importa por CSV" }
                            span { "✓ Ley 21.719 incluida" }
                        }
                    }
                    div { class: "hero-visual",
                        img { class: "hero-img", src: "{HERO_IMG}", alt: "Equipo comercial trabajando" }
                        div { class: "float-card float-a",
                            strong { "+$48.500" }
                            span { "ganado este mes" }
                        }
                        div { class: "float-card float-b",
                            strong { "12" }
                            span { "negocios en negociación" }
                        }
                    }
                }
            }

            section { class: "stats-band",
                div { class: "stat-light",
                    strong { "8+" }
                    span { "módulos integrados" }
                }
                div { class: "stat-light",
                    strong { "6" }
                    span { "etapas de venta" }
                }
                div { class: "stat-light",
                    strong { "100%" }
                    span { "datos bajo tu control" }
                }
                div { class: "stat-light",
                    strong { "24/7" }
                    span { "tu operación en línea" }
                }
            }

            section { class: "section",
                p { class: "eyebrow eyebrow-center", "Módulos" }
                h2 { class: "section-title section-center", "Un sistema, todo el ciclo comercial" }
                p { class: "subtitle subtitle-center",
                    "Desde el primer contacto hasta el cierre y la postventa."
                }
                div { class: "cards-media",
                    div { class: "card card-media",
                        img { src: "{MEET_IMG}", alt: "Captura de prospectos" }
                        div { class: "card-media-body",
                            h3 { "Prospectos" }
                            p { "Califica por puntaje y convierte a contacto, empresa y negocio con un clic." }
                            Link { to: Route::Leads {}, class: "link", "Ver prospectos →" }
                        }
                    }
                    div { class: "card card-media",
                        img { src: "{TEAM_IMG}", alt: "Equipo junto a clientes" }
                        div { class: "card-media-body",
                            h3 { "Contactos y empresas" }
                            p { "Fichas completas con etiquetas, actividad ligada e ingresos por empresa." }
                            Link { to: Route::Contacts {}, class: "link", "Ver contactos →" }
                        }
                    }
                    div { class: "card card-media",
                        img { src: "{DEAL_IMG}", alt: "Cierre de negocio" }
                        div { class: "card-media-body",
                            h3 { "Negocios" }
                            p { "Tablero kanban de 6 etapas con valores, cierres e importación CSV." }
                            Link { to: Route::Deals {}, class: "link", "Ver tablero →" }
                        }
                    }
                    div { class: "card card-media",
                        img { src: "{DATA_IMG}", alt: "Análisis de datos" }
                        div { class: "card-media-body",
                            h3 { "Actividades e informes" }
                            p { "Tareas con vencimientos, calendario sincronizado y reportes de pipeline." }
                            Link { to: Route::Activities {}, class: "link", "Ver actividades →" }
                        }
                    }
                    div { class: "card card-media",
                        img { src: "{OFFICE_IMG}", alt: "Oficina moderna" }
                        div { class: "card-media-body",
                            h3 { "Documentos" }
                            p { "Sube, descarga y organiza los archivos de cada negocio y cliente." }
                            Link { to: Route::Documents {}, class: "link", "Ver documentos →" }
                        }
                    }
                    div { class: "card card-media",
                        img { src: "{HERO_IMG}", alt: "Equipo comercial" }
                        div { class: "card-media-body",
                            h3 { "Comunicación total" }
                            p { "Email con plantillas, WhatsApp por contacto y notificaciones." }
                            Link { to: Route::Reports {}, class: "link", "Ver informes →" }
                        }
                    }
                }
            }

            section { class: "section",
                p { class: "eyebrow eyebrow-center", "Por qué OCCB" }
                h2 { class: "section-title section-center", "Hecho para equipos que venden" }
                div { class: "feature-row",
                    img { class: "feature-img", src: "{TEAM_IMG}", alt: "Equipo trabajando junto" }
                    div {
                        span { class: "step-num", "01" }
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
                        span { class: "step-num", "02" }
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
                        span { class: "step-num", "03" }
                        h3 { "Comunicación integrada" }
                        p { class: "muted",
                            "Email con plantillas y bitácora, WhatsApp con hilos por contacto y "
                            "calendario sincronizado con Google."
                        }
                    }
                }
            }

            section { class: "steps-band",
                p { class: "eyebrow eyebrow-center eyebrow-light", "Cómo empezar" }
                h2 { class: "section-title section-center section-light", "Empieza en 3 pasos" }
                div { class: "steps-grid",
                    div { class: "step",
                        span { class: "step-dot", "1" }
                        h3 { "Crea tu cuenta" }
                        p { "Regístrate en un minuto. La primera cuenta queda como administradora." }
                    }
                    div { class: "step",
                        span { class: "step-dot", "2" }
                        h3 { "Carga tus datos" }
                        p { "Importa tus contactos y empresas por CSV o créalos a mano." }
                    }
                    div { class: "step",
                        span { class: "step-dot", "3" }
                        h3 { "Vende y mide" }
                        p { "Mueve negocios por el tablero y sigue el avance en informes." }
                    }
                }
                div { class: "row steps-cta",
                    Link {
                        to: Route::Register {},
                        class: "btn btn-large btn-white",
                        "Crear cuenta gratis"
                    }
                    Link { to: Route::Login {}, class: "btn btn-large btn-outline-light", "Entrar" }
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
