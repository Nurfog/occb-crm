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
                p { class: "mt-4 flex items-center gap-2 text-sm text-slate-300",
                    span { class: "dot dot-ok" }
                    "{tag}"
                }
            }
        }
        _ => rsx! {},
    };

    rsx! {
        Layout { title: "".to_string(),
            // HERO
            section { class: "relative overflow-hidden rounded-3xl bg-slate-950",
                div { class: "pointer-events-none absolute -top-32 -right-24 h-96 w-96 rounded-full bg-[#0176D3]/30 blur-3xl" }
                div { class: "pointer-events-none absolute -bottom-40 -left-24 h-96 w-96 rounded-full bg-violet-600/20 blur-3xl" }
                div { class: "relative grid items-center gap-10 p-8 md:grid-cols-2 md:p-12",
                    div {
                        p { class: "mb-3 text-xs font-extrabold tracking-[0.2em] text-[#7CC4F8]",
                            "OCCB · CRM"
                        }
                        h1 { class: "text-4xl leading-tight font-extrabold text-white md:text-5xl",
                            "Toda tu relación con clientes, "
                            span { class: "bg-gradient-to-r from-[#1B96FF] to-violet-400 bg-clip-text text-transparent",
                                "en un solo lugar"
                            }
                        }
                        p { class: "mt-4 max-w-xl text-lg text-slate-300",
                            "Prospectos, contactos, empresas y negocios con tablero kanban, "
                            "actividades, documentos, informes y comunicación integrada. "
                            "Tus datos en tu infraestructura, bajo Ley 21.719."
                        }
                        div { class: "mt-7 flex flex-wrap gap-3",
                            Link {
                                to: Route::Register {},
                                class: "rounded-xl bg-[#0176D3] px-6 py-3 text-base font-semibold text-white shadow-lg shadow-[#0176D3]/40 transition hover:-translate-y-0.5 hover:bg-[#014486]",
                                "Crear cuenta gratis"
                            }
                            Link {
                                to: Route::Login {},
                                class: "rounded-xl border border-white/25 px-6 py-3 text-base font-semibold text-white transition hover:bg-white/10",
                                "Entrar"
                            }
                        }
                        {status_pill}
                        div { class: "mt-5 flex flex-wrap gap-x-5 gap-y-1 text-sm text-slate-400",
                            span { "✓ Sin tarjetas" }
                            span { "✓ Importa por CSV" }
                            span { "✓ Ley 21.719 incluida" }
                        }
                    }
                    div { class: "relative",
                        img {
                            class: "aspect-[4/3.4] w-full rounded-2xl object-cover shadow-2xl shadow-black/50",
                            src: "{HERO_IMG}",
                            alt: "Equipo comercial trabajando"
                        }
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

            // STATS
            section { class: "mt-6 grid grid-cols-2 gap-4 rounded-2xl bg-gradient-to-r from-[#032D60] to-[#0176D3] p-6 text-white shadow-xl shadow-[#0176D3]/20 md:grid-cols-4",
                div { class: "text-center",
                    p { class: "text-3xl font-extrabold", "8+" }
                    p { class: "text-sm text-[#D4E9F9]", "módulos integrados" }
                }
                div { class: "text-center",
                    p { class: "text-3xl font-extrabold", "6" }
                    p { class: "text-sm text-[#D4E9F9]", "etapas de venta" }
                }
                div { class: "text-center",
                    p { class: "text-3xl font-extrabold", "100%" }
                    p { class: "text-sm text-[#D4E9F9]", "datos bajo tu control" }
                }
                div { class: "text-center",
                    p { class: "text-3xl font-extrabold", "24/7" }
                    p { class: "text-sm text-[#D4E9F9]", "tu operación en línea" }
                }
            }

            // MÓDULOS
            section { class: "mt-14",
                p { class: "text-center text-xs font-extrabold tracking-[0.2em] text-[#0176D3]",
                    "MÓDULOS"
                }
                h2 { class: "mt-1 text-center text-3xl font-extrabold tracking-tight",
                    "Un sistema, todo el ciclo comercial"
                }
                p { class: "mx-auto mt-2 max-w-2xl text-center text-slate-500",
                    "Desde el primer contacto hasta el cierre y la postventa."
                }
                div { class: "mt-8 grid gap-5 sm:grid-cols-2 lg:grid-cols-3",
                    ModuleCard {
                        img: MEET_IMG,
                        alt: "Captura de prospectos",
                        title: "Prospectos",
                        desc: "Califica por puntaje y convierte a contacto, empresa y negocio con un clic.",
                        to: Route::Leads {},
                        cta: "Ver prospectos →",
                    }
                    ModuleCard {
                        img: TEAM_IMG,
                        alt: "Equipo junto a clientes",
                        title: "Contactos y empresas",
                        desc: "Fichas completas con etiquetas, actividad ligada e ingresos por empresa.",
                        to: Route::Contacts {},
                        cta: "Ver contactos →",
                    }
                    ModuleCard {
                        img: DEAL_IMG,
                        alt: "Cierre de negocio",
                        title: "Negocios",
                        desc: "Tablero kanban de 6 etapas con valores, cierres e importación CSV.",
                        to: Route::Deals {},
                        cta: "Ver tablero →",
                    }
                    ModuleCard {
                        img: DATA_IMG,
                        alt: "Análisis de datos",
                        title: "Actividades e informes",
                        desc: "Tareas con vencimientos, calendario sincronizado y reportes de pipeline.",
                        to: Route::Activities {},
                        cta: "Ver actividades →",
                    }
                    ModuleCard {
                        img: OFFICE_IMG,
                        alt: "Oficina moderna",
                        title: "Documentos",
                        desc: "Sube, descarga y organiza los archivos de cada negocio y cliente.",
                        to: Route::Documents {},
                        cta: "Ver documentos →",
                    }
                    ModuleCard {
                        img: HERO_IMG,
                        alt: "Equipo comercial",
                        title: "Comunicación total",
                        desc: "Email con plantillas, WhatsApp por contacto y notificaciones.",
                        to: Route::Reports {},
                        cta: "Ver informes →",
                    }
                }
            }

            // POR QUÉ
            section { class: "mt-14",
                p { class: "text-center text-xs font-extrabold tracking-[0.2em] text-[#0176D3]",
                    "POR QUÉ OCCB"
                }
                h2 { class: "mt-1 text-center text-3xl font-extrabold tracking-tight",
                    "Hecho para equipos que venden"
                }
                div { class: "mt-8 space-y-5",
                    FeatureRow {
                        img: TEAM_IMG,
                        alt: "Equipo trabajando junto",
                        num: "01",
                        title: "Trabajo en equipo",
                        desc: "Reparto automático de prospectos, notificaciones, bitácora de actividad y permisos por perfil: cada persona ve lo que le toca.",
                        reverse: false,
                    }
                    FeatureRow {
                        img: DATA_IMG,
                        alt: "Análisis de datos en un computador",
                        num: "02",
                        title: "Decisiones con datos",
                        desc: "Estadísticas del negocio, mejores negocios, conversión de prospectos y reportes exportables. Sin planillas paralelas.",
                        reverse: true,
                    }
                    FeatureRow {
                        img: MEET_IMG,
                        alt: "Reunión de planificación",
                        num: "03",
                        title: "Comunicación integrada",
                        desc: "Email con plantillas y bitácora, WhatsApp con hilos por contacto y calendario sincronizado con Google.",
                        reverse: false,
                    }
                }
            }

            // PASOS
            section { class: "relative mt-14 overflow-hidden rounded-3xl bg-gradient-to-br from-slate-950 via-slate-900 to-[#032D60] p-8 text-white md:p-12",
                p { class: "text-center text-xs font-extrabold tracking-[0.2em] text-[#7CC4F8]",
                    "CÓMO EMPEZAR"
                }
                h2 { class: "mt-1 text-center text-3xl font-extrabold tracking-tight",
                    "Empieza en 3 pasos"
                }
                div { class: "mt-8 grid gap-4 md:grid-cols-3",
                    div { class: "rounded-2xl border border-white/15 bg-white/5 p-6 backdrop-blur",
                        span { class: "inline-flex h-9 w-9 items-center justify-center rounded-full bg-white font-extrabold text-[#014486]",
                            "1"
                        }
                        h3 { class: "mt-3 text-lg font-bold", "Crea tu cuenta" }
                        p { class: "mt-1 text-sm text-slate-300",
                            "Regístrate en un minuto. La primera cuenta queda como administradora."
                        }
                    }
                    div { class: "rounded-2xl border border-white/15 bg-white/5 p-6 backdrop-blur",
                        span { class: "inline-flex h-9 w-9 items-center justify-center rounded-full bg-white font-extrabold text-[#014486]",
                            "2"
                        }
                        h3 { class: "mt-3 text-lg font-bold", "Carga tus datos" }
                        p { class: "mt-1 text-sm text-slate-300",
                            "Importa tus contactos y empresas por CSV o créalos a mano."
                        }
                    }
                    div { class: "rounded-2xl border border-white/15 bg-white/5 p-6 backdrop-blur",
                        span { class: "inline-flex h-9 w-9 items-center justify-center rounded-full bg-white font-extrabold text-[#014486]",
                            "3"
                        }
                        h3 { class: "mt-3 text-lg font-bold", "Vende y mide" }
                        p { class: "mt-1 text-sm text-slate-300",
                            "Mueve negocios por el tablero y sigue el avance en informes."
                        }
                    }
                }
                div { class: "mt-8 flex flex-wrap justify-center gap-3",
                    Link {
                        to: Route::Register {},
                        class: "rounded-xl bg-white px-6 py-3 text-base font-semibold text-slate-950 transition hover:-translate-y-0.5",
                        "Crear cuenta gratis"
                    }
                    Link {
                        to: Route::Login {},
                        class: "rounded-xl border border-white/25 px-6 py-3 text-base font-semibold text-white transition hover:bg-white/10",
                        "Entrar"
                    }
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

/// Tarjeta de módulo con foto.
#[component]
fn ModuleCard(
    img: String,
    alt: String,
    title: String,
    desc: String,
    to: Route,
    cta: String,
) -> Element {
    rsx! {
        div { class: "group overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm transition duration-200 hover:-translate-y-1 hover:shadow-xl",
            img { class: "h-40 w-full bg-slate-200 object-cover", src: "{img}", alt: "{alt}" }
            div { class: "p-5",
                h3 { class: "text-lg font-bold", "{title}" }
                p { class: "mt-1 text-sm text-slate-500", "{desc}" }
                p { class: "mt-3",
                    Link { to: to, class: "link text-sm font-semibold", "{cta}" }
                }
            }
        }
    }
}

/// Fila alternada imagen/texto.
#[component]
fn FeatureRow(img: String, alt: String, num: String, title: String, desc: String, reverse: bool) -> Element {
    let dir = if reverse { "md:flex-row-reverse" } else { "" };
    rsx! {
        div { class: "flex flex-col items-center gap-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm md:flex-row {dir}",
            img { class: "w-full rounded-xl object-cover md:w-72 aspect-[4/3] bg-slate-200", src: "{img}", alt: "{alt}" }
            div {
                span { class: "inline-block rounded-full bg-[#E8F3FC] px-3 py-0.5 text-xs font-extrabold text-[#014486]",
                    "{num}"
                }
                h3 { class: "mt-2 text-xl font-bold", "{title}" }
                p { class: "mt-1 text-slate-500", "{desc}" }
            }
        }
    }
}
