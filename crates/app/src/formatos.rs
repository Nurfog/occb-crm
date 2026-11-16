//! Formato y etiquetas puras del frontal (sin estado global propio).
//! Extraído de `main.rs` sin cambios de conducta (higiene M0): fechas UTC,
//! etiquetas ES/EN, salud, frase de reglas, títulos de actividad,
//! selección masiva, recargas con respaldo demo y helpers de territorio.
//! Las vistas siguen en `main.rs`.

use dioxus::prelude::*;
use uuid::Uuid;
use occb_core::{
    Automatizacion, Colegio, EstadoAlumno, EstadoTarea, EtapaOportunidad, Evento, Idioma,
    Oportunidad, Persona, textos,
};

use crate::cliente;
use occb_core::{datos_semilla_colegios, datos_semilla_oportunidades, datos_semilla_personas};

/// Fecha relativa estilo Twenty ("hace 39 minutos" / "39 minutes ago").
pub fn hace_tiempo(fecha: chrono::DateTime<chrono::Utc>, idioma: Idioma) -> String {
    let mins = (chrono::Utc::now() - fecha).num_minutes().max(0);
    if idioma == Idioma::Es {
        if mins < 1 {
            "ahora mismo".to_string()
        } else if mins < 60 {
            format!("hace {mins} minutos")
        } else if mins < 60 * 24 {
            format!("hace {} horas", mins / 60)
        } else {
            format!("hace {} días", mins / 60 / 24)
        }
    } else if mins < 1 {
        "just now".to_string()
    } else if mins < 60 {
        format!("{mins} minutes ago")
    } else if mins < 60 * 24 {
        format!("{} hours ago", mins / 60)
    } else {
        format!("{} days ago", mins / 60 / 24)
    }
}

/// Etiqueta de etapa según el idioma (la BD guarda siempre el texto español).
pub fn etiqueta_etapa(etapa: EtapaOportunidad, idioma: Idioma) -> &'static str {
    match (etapa, idioma) {
        (EtapaOportunidad::Contacto, Idioma::Es) => "Contacto",
        (EtapaOportunidad::Postulante, Idioma::Es) => "Postulante",
        (EtapaOportunidad::Evaluado, Idioma::Es) => "Evaluado",
        (EtapaOportunidad::Aceptado, Idioma::Es) => "Aceptado",
        (EtapaOportunidad::Matriculado, Idioma::Es) => "Matriculado",
        (EtapaOportunidad::Contacto, Idioma::En) => "Contact",
        (EtapaOportunidad::Postulante, Idioma::En) => "Applicant",
        (EtapaOportunidad::Evaluado, Idioma::En) => "Evaluated",
        (EtapaOportunidad::Aceptado, Idioma::En) => "Accepted",
        (EtapaOportunidad::Matriculado, Idioma::En) => "Enrolled",
    }
}

pub fn etiqueta_tipo_busqueda(tipo: &str, idioma: Idioma) -> &'static str {
    match (tipo, idioma) {
        ("alumno", Idioma::Es) => "Alumnos",
        ("alumno", Idioma::En) => "Students",
        ("apoderado", Idioma::Es) => "Apoderados",
        ("apoderado", Idioma::En) => "Guardians",
        ("persona", Idioma::Es) => "Personas",
        ("persona", Idioma::En) => "People",
        ("postulacion", Idioma::Es) => "Admisión",
        ("postulacion", Idioma::En) => "Admissions",
        ("tarea", Idioma::Es) => "Tareas",
        ("tarea", Idioma::En) => "Tasks",
        ("nota", Idioma::Es) => "Notas",
        ("nota", Idioma::En) => "Notes",
        ("evento", Idioma::Es) => "Eventos",
        ("evento", Idioma::En) => "Events",
        ("correo", Idioma::Es) => "Correos",
        ("correo", Idioma::En) => "Mail",
        ("colegio", Idioma::Es) => "Colegios",
        ("colegio", Idioma::En) => "Schools",
        _ => "",
    }
}

/// Frase humana de una regla (S3, solo UI): Cuando/Si/Entonces desde el JSON.
/// Sin condición muestra texto fijo; clave desconocida muestra su nombre.
pub fn frase_regla(regla: &Automatizacion, idioma: Idioma) -> String {
    let t = textos(idioma);
    let y = if idioma == Idioma::Es { " y " } else { " and " };
    let conds = regla
        .condiciones
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| {
                    let campo = c.get("campo").and_then(|x| x.as_str()).unwrap_or("?");
                    let op = c.get("operador").and_then(|x| x.as_str()).unwrap_or("?");
                    let val = c.get("valor").and_then(|x| x.as_str()).unwrap_or("?");
                    format!("{campo} {op} {val}")
                })
                .collect::<Vec<_>>()
                .join(y)
        })
        .unwrap_or_default();
    let conds = if conds.is_empty() {
        if idioma == Idioma::Es {
            "Sin condición (siempre)".to_string()
        } else {
            "No condition (always)".to_string()
        }
    } else {
        conds
    };
    let acts = regla
        .acciones
        .as_array()
        .map(|a| {
            a.iter()
                .map(|x| x.get("tipo").and_then(|v| v.as_str()).unwrap_or("?").to_string())
                .collect::<Vec<_>>()
                .join(y)
        })
        .unwrap_or_default();
    let acts = if acts.is_empty() {
        if idioma == Idioma::Es {
            "Sin acción".to_string()
        } else {
            "No action".to_string()
        }
    } else {
        acts
    };
    format!("{} {} · {} {} · {} {}", t.cuando, regla.disparador, t.si, conds, t.entonces, acts)
}

pub fn titulo_actividad(actividad: &occb_core::Actividad) -> &str {
    actividad
        .detalle
        .get("titulo")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Registro")
}

/// Etiqueta de estado de alumno según idioma.
pub fn etiqueta_estado(estado: EstadoAlumno, idioma: Idioma) -> &'static str {
    match (estado, idioma) {
        (EstadoAlumno::Postulante, Idioma::Es) => "Postulante",
        (EstadoAlumno::Matriculado, Idioma::Es) => "Matriculado",
        (EstadoAlumno::Trasladado, Idioma::Es) => "Trasladado",
        (EstadoAlumno::Retirado, Idioma::Es) => "Retirado",
        (EstadoAlumno::Postulante, Idioma::En) => "Applicant",
        (EstadoAlumno::Matriculado, Idioma::En) => "Enrolled",
        (EstadoAlumno::Trasladado, Idioma::En) => "Transferred",
        (EstadoAlumno::Retirado, Idioma::En) => "Withdrawn",
    }
}

/// Iniciales para avatares.
pub fn iniciales_de(nombres: &str, apellidos: &str) -> String {
    format!(
        "{}{}",
        nombres.chars().next().unwrap_or('?'),
        apellidos.chars().next().unwrap_or('?')
    )
}

/// Alterna un id en una selección (los `Signal` se copian).
pub fn alternar_seleccion(mut seleccion: Signal<Vec<Uuid>>, id: Uuid) {
    let mut actual = seleccion();
    if actual.contains(&id) {
        actual.retain(|x| x != &id);
    } else {
        actual.push(id);
    }
    seleccion.set(actual);
}

/// Marca todas las filas o limpia si ya estaban todas marcadas.
pub fn alternar_todos(mut seleccion: Signal<Vec<Uuid>>, ids: Vec<Uuid>) {
    let actual = seleccion();
    if !ids.is_empty() && ids.iter().all(|id| actual.contains(id)) {
        seleccion.set(Vec::new());
    } else {
        seleccion.set(ids);
    }
}

/// Sección de salud reutilizada en el alta y la edición de alumno.
/// Los `Signal` se copian; el llamador conserva la propiedad.
pub fn campos_salud(
    idioma: Idioma,
    mut grupo: Signal<String>,
    mut alergias: Signal<String>,
    mut medicamentos: Signal<String>,
    mut antecedentes: Signal<String>,
    mut emergencia: Signal<String>,
) -> Element {
    rsx! {
        div { class: "campo",
            label { if idioma == Idioma::Es { "Grupo sanguíneo" } else { "Blood type" } }
            select {
                class: "input",
                value: "{grupo}",
                onchange: move |e| grupo.set(e.value()),
                option { value: "", "—" }
                for g in ["O-", "O+", "A-", "A+", "B-", "B+", "AB-", "AB+"] {
                    option { value: "{g}", "{g}" }
                }
            }
        }
        div { class: "campo",
            label { if idioma == Idioma::Es { "Alergias" } else { "Allergies" } }
            input { class: "input", value: "{alergias}", oninput: move |e| alergias.set(e.value()) }
        }
        div { class: "campo",
            label { if idioma == Idioma::Es { "Medicamentos" } else { "Medications" } }
            input { class: "input", value: "{medicamentos}", oninput: move |e| medicamentos.set(e.value()) }
        }
        div { class: "campo",
            label { if idioma == Idioma::Es { "Antecedentes médicos" } else { "Medical history" } }
            input { class: "input", value: "{antecedentes}", oninput: move |e| antecedentes.set(e.value()) }
        }
        div { class: "campo",
            label { if idioma == Idioma::Es { "Contacto de emergencia" } else { "Emergency contact" } }
            input { class: "input", value: "{emergencia}", oninput: move |e| emergencia.set(e.value()) }
        }
    }
}

/// Panel de lectura con los datos de salud de la ficha del alumno.
pub fn panel_salud(
    idioma: Idioma,
    titulo: String,
    grupo: String,
    alergias: String,
    medicamentos: String,
    antecedentes: String,
    emergencia: String,
) -> Element {
    rsx! {
        div { class: "panel",
            div { class: "panel-head",
                span { class: "panel-title", "{titulo}" }
                span { class: "link", "↗" }
            }
            div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "🩸 Grupo" } else { "🩸 Type" } } span { class: "mini-name", "{grupo}" } }
            div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "⚠ Alergias" } else { "⚠ Allergies" } } span { class: "mini-name", "{alergias}" } }
            div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "💊 Medicamentos" } else { "💊 Medications" } } span { class: "mini-name", "{medicamentos}" } }
            div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "📋 Antecedentes" } else { "📋 History" } } span { class: "mini-name", "{antecedentes}" } }
            div { class: "mini-row", span { class: "muted", if idioma == Idioma::Es { "📞 Emergencia" } else { "📞 Emergency" } } span { class: "mini-name", "{emergencia}" } }
        }
    }
}

/// Etiqueta de estado de tarea según idioma.
pub fn etiqueta_tarea(estado: EstadoTarea, idioma: Idioma) -> &'static str {
    match (estado, idioma) {
        (EstadoTarea::Pendiente, Idioma::Es) => "Pendiente",
        (EstadoTarea::EnCurso, Idioma::Es) => "En curso",
        (EstadoTarea::Hecha, Idioma::Es) => "Hecha",
        (EstadoTarea::Pendiente, Idioma::En) => "To do",
        (EstadoTarea::EnCurso, Idioma::En) => "In progress",
        (EstadoTarea::Hecha, Idioma::En) => "Done",
    }
}

/// Convención horaria del frontal: los `datetime-local` se guardan y muestran
/// en UTC (van y vuelven idénticos; el colegio opera en un solo huso).
pub fn a_campo_local(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%Y-%m-%dT%H:%M").to_string()
}

/// Hora corta (`15:04`) para eventos y correos.
pub fn fmt_hora(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%H:%M UTC").to_string()
}

/// Día y hora (`20/11 15:04 UTC`) para relacionados y listas.
/// Día y hora con sufijo UTC estático de una sola vía (`20/11 15:04 UTC`).
/// Nunca va en `data-dia`, `.cal-punto`, `.opp-name`, `.stage` ni `[data-correo]`.
pub fn fmt_dia_hora(fecha: chrono::DateTime<chrono::Utc>) -> String {
    fecha.format("%d/%m %H:%M UTC").to_string()
}

/// Fecha completa (`20/11/2026`) para el día seleccionado.
pub fn fmt_fecha(fecha: chrono::NaiveDate) -> String {
    fecha.format("%d/%m/%Y").to_string()
}

/// Lee un `datetime-local` (`YYYY-MM-DDTHH:MM`) como UTC.
pub fn desde_campo_local(valor: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::NaiveDateTime::parse_from_str(&format!("{valor}:00"), "%Y-%m-%dT%H:%M:%S")
        .ok()
        .map(|ingenuo| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ingenuo, chrono::Utc))
}

/// Recarga los eventos del mes indicado en las señales dadas.
pub fn recargar_mes(
    ano: Signal<i32>,
    mes: Signal<u32>,
    mut eventos: Signal<Vec<Evento>>,
    mut aviso: Signal<String>,
) {
    let (a, m) = (ano(), mes());
    spawn(async move {
        match cliente::eventos_mes(a, m).await {
            Ok(lista) => {
                aviso.set(String::new());
                eventos.set(lista);
            }
            Err(mensaje) => aviso.set(mensaje),
        }
    });
}

/// Mueve el mes visible y deja el día 1 seleccionado.
pub fn mover_mes(
    delta: i32,
    mut ano: Signal<i32>,
    mut mes: Signal<u32>,
    mut dia_sel: Signal<chrono::NaiveDate>,
    hoy: chrono::NaiveDate,
) {
    let (mut a, mut m) = (ano(), mes());
    if delta < 0 {
        if m == 1 {
            a -= 1;
            m = 12;
        } else {
            m -= 1;
        }
    } else if m == 12 {
        a += 1;
        m = 1;
    } else {
        m += 1;
    }
    ano.set(a);
    mes.set(m);
    dia_sel.set(chrono::NaiveDate::from_ymd_opt(a, m, 1).unwrap_or(hoy));
}

/// Nombres de mes según idioma.
pub fn nombre_mes(mes: u32, idioma: Idioma) -> &'static str {
    match (mes, idioma) {
        (1, Idioma::Es) => "enero",
        (2, Idioma::Es) => "febrero",
        (3, Idioma::Es) => "marzo",
        (4, Idioma::Es) => "abril",
        (5, Idioma::Es) => "mayo",
        (6, Idioma::Es) => "junio",
        (7, Idioma::Es) => "julio",
        (8, Idioma::Es) => "agosto",
        (9, Idioma::Es) => "septiembre",
        (10, Idioma::Es) => "octubre",
        (11, Idioma::Es) => "noviembre",
        (12, Idioma::Es) => "diciembre",
        (1, Idioma::En) => "January",
        (2, Idioma::En) => "February",
        (3, Idioma::En) => "March",
        (4, Idioma::En) => "April",
        (5, Idioma::En) => "May",
        (6, Idioma::En) => "June",
        (7, Idioma::En) => "July",
        (8, Idioma::En) => "August",
        (9, Idioma::En) => "September",
        (10, Idioma::En) => "October",
        (11, Idioma::En) => "November",
        (12, Idioma::En) => "December",
        _ => "?",
    }
}

/// Código de región desde el selector (1–16) o `None`.
pub fn region_codigo_de(sel: &Signal<String>) -> Option<i16> {
    sel.read().parse::<i16>().ok().filter(|r| (1..=16).contains(r))
}

/// CUT de comuna desde el selector o `None` si está vacío.
pub fn comuna_codigo_de(sel: &Signal<String>) -> Option<String> {
    let texto = sel.read().clone();
    if texto.trim().is_empty() {
        None
    } else {
        Some(texto)
    }
}

/// Ciudad para guardar: el nombre oficial de la comuna elegida (o vacío).
pub fn ciudad_de_comuna(sel: &Signal<String>) -> String {
    let cut = sel.read().clone();
    if cut.trim().is_empty() {
        String::new()
    } else {
        occb_core::territorio::comuna_por_cut(cut.trim())
            .map(|c| c.nombre.to_string())
            .unwrap_or_default()
    }
}

/// Colegios de demostración (el directorio oficial vive en PG con 12.038).
pub fn cargar_colegios_demo() -> Vec<Colegio> {
    datos_semilla_colegios()
}

/// Personas de demostración.
pub fn cargar_personas_demo() -> Vec<Persona> {
    datos_semilla_personas()
}

/// Recarga personas desde la API; usa demo solo si aún no hay nada.
pub fn recargar_personas(mut personas: Signal<Vec<Persona>>) {
    spawn(async move {
        match cliente::listar_personas().await {
            Ok(lista) => personas.set(lista),
            Err(_) if personas.read().is_empty() => personas.set(cargar_personas_demo()),
            Err(_) => {}
        }
    });
}

/// Recarga oportunidades desde la API; usa demo solo si aún no hay nada.
pub fn recargar_oportunidades(mut oportunidades: Signal<Vec<Oportunidad>>) {
    spawn(async move {
        match cliente::listar_oportunidades().await {
            Ok(lista) => oportunidades.set(lista),
            Err(_) if oportunidades.read().is_empty() => {
                let demo_p = cargar_personas_demo();
                let ids: Vec<_> = demo_p.iter().map(|p| p.id).collect();
                oportunidades.set(datos_semilla_oportunidades(&ids));
            }
            Err(_) => {}
        }
    });
}
