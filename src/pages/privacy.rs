//! Política de Privacidad (Ley 21.719, Chile).
//!
//! Datos del responsable provistos por el equipo (entorno de desarrollo).
//! Revisar con abogado antes de publicar en producción.

use dioxus::prelude::*;

use crate::{components::Layout, privacy::privacy_contact_email, route::Route};

#[component]
pub fn Privacy() -> Element {
    let contact = privacy_contact_email();
    rsx! {
        Layout { title: "Política de Privacidad".to_string(),
            p { class: "subtitle",
                "Cómo tratamos tus datos personales según la Ley 21.719 de Chile. "
                "Vigencia de la ley: 1 de diciembre de 2026."
            }

            section { class: "card",
                h2 { class: "card-title", "1. Quién es responsable de tus datos" }
                p {
                    "Responsable del tratamiento: prueba, domicilio 1-9 mi casa 123. "
                    "[PENDIENTE: agregar RUT del responsable]. "
                    "Contacto de privacidad: "
                    strong { "{contact}" }
                    ". Delegado de Protección de Datos: juan."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "2. Qué datos recogemos" }
                ul {
                    li { "Cuenta: nombre, email y clave (guardamos solo su hash, nunca la clave)." }
                    li { "Sesión: tokens de acceso y renovación, solo en memoria de tu navegador." }
                    li { "Soporte y derechos: los datos que incluyas en tus solicitudes." }
                    li { "Opcionales con tu consentimiento: preferencias de marketing y analítica." }
                }
                p { class: "muted small",
                    "No pedimos datos sensibles ni datos de niños, niñas o adolescentes. "
                    "Si detectamos alguno, lo eliminamos."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "3. Para qué los usamos (finalidad y base)" }
                ul {
                    li {
                        "Crear y mantener tu cuenta y tu sesión — base: el contrato (términos del servicio)."
                    }
                    li {
                        "Seguridad y prevención de abuso (límites de intentos, registros) — base: interés legítimo y obligación legal."
                    }
                    li {
                        "Marketing directo — base: solo tu consentimiento previo y expreso, revocable cuando quieras."
                    }
                    li {
                        "Analítica de uso — base: solo tu consentimiento previo y expreso, revocable cuando quieras."
                    }
                }
                p { class: "muted small",
                    "Nunca usamos tus datos para una finalidad distinta sin informarte y, cuando la ley lo exige, pedirte un nuevo consentimiento."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "4. Tu consentimiento" }
                p {
                    "Las finalidades opcionales están desactivadas por defecto. Puedes aceptarlas o rechazarlas "
                    "en el banner, por separado, y cambiar de opinión cuando quieras en "
                    "«Preferencias de privacidad» al pie de cada página. Revocar es tan fácil como aceptar."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "5. Cuánto tiempo los guardamos" }
                p {
                    "Conservamos tus datos mientras tu cuenta esté activa y hasta 15 años "
                    "después de cerrarla, por obligaciones legales. Cumplido el plazo, "
                    "los datos se eliminan o se anonimizan."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "6. Con quién los compartimos" }
                p {
                    "Estamos en desarrollo: por ahora no hay encargados externos ni "
                    "transferencias internacionales. Si se incorporan (p. ej. hosting, "
                    "correo, analítica), se informarán aquí con sus garantías. "
                    "No vendemos tus datos personales."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "7. Tus derechos (ARCOP+B)" }
                p {
                    "Tienes derecho a Acceso, Rectificación, Cancelación/supresión, Oposición, Portabilidad y "
                    "Bloqueo temporal. Respondemos en 30 días corridos (prorrogables 30 más con aviso) y el "
                    "bloqueo temporal en 2 días hábiles. Ejerce tus derechos gratis en "
                    Link { to: Route::Rights {}, class: "link", "esta página" }
                    " o escribiendo a "
                    strong { "{contact}" }
                    "."
                }
                p { class: "muted small",
                    "Si nuestra respuesta no te deja conforme, puedes reclamar ante la Agencia de Protección "
                    "de Datos Personales."
                }
            }

            section { class: "card",
                h2 { class: "card-title", "8. Cómo los protegemos" }
                ul {
                    li { "Claves con hash robusto y tokens de corta duración con rotación." }
                    li { "Tokens solo en memoria del navegador, nunca en almacenamiento persistente." }
                    li { "Conexiones cifradas (HTTPS) y accesos con mínimo privilegio." }
                    li {
                        "Ante una brecha que te afecte, te avisaremos y notificaremos a la Agencia según la ley."
                    }
                }
            }

            p { class: "muted small",
                "Última actualización: 2026-10-19. Te avisaremos de cambios relevantes antes de que apliquen."
            }
        }
    }
}
