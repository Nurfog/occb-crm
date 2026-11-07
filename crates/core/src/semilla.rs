//! Datos de demostración en español con relaciones coherentes.
//! Se usan en el frontal (modo sin conexión) y como referencia de `sembrar_datos_demo()` en SQL.

use super::dominio::{
    Alumno, Apoderado, Colegio, EstadoAlumno, EstadoTarea, EtapaOportunidad, Nota, Oportunidad,
    Persona, Tarea,
};
use chrono::Utc;
use uuid::Uuid;

/// Cuatro colegios de ejemplo (el directorio oficial vive en PG con 12.038).
pub fn datos_semilla_colegios() -> Vec<Colegio> {
    let crear = |rbd: &str, nombre: &str, dependencia: &str, region: i16, comuna: &str| Colegio {
        rbd: rbd.into(),
        dv: String::new(),
        nombre: nombre.into(),
        dependencia: dependencia.into(),
        region,
        comuna: comuna.into(),
        latitud: None,
        longitud: None,
        estado: "Vigente".into(),
    };
    let mut lista = vec![
        crear("10001", "Liceo Bicentenario Demo", "Municipal", 13, "13101"),
        crear("10002", "Colegio San Demo", "Particular Subvencionado", 5, "05101"),
        crear("10003", "Escuela Rural Demo", "Municipal", 8, "08101"),
        crear("10004", "Liceo Técnico Demo", "Particular Pagado", 13, "13123"),
    ];
    lista.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    lista
}

/// Cinco personas de ejemplo con territorio chileno.
pub fn datos_semilla_personas() -> Vec<Persona> {
    let ahora = Utc::now();
    let crear = |nombre: &str,
                 apellido: &str,
                 correo: &str,
                 puesto: &str,
                 region: i16,
                 comuna: &str| Persona {
        id: Uuid::new_v4(),
        nombre: nombre.into(),
        apellido: apellido.into(),
        correo: correo.into(),
        telefono: "+56 9 5555 0100".into(),
        puesto: puesto.into(),
        ciudad: crate::territorio::comuna_por_cut(comuna)
            .map(|c| c.nombre.to_string())
            .unwrap_or_default(),
        region_codigo: Some(region),
        comuna_codigo: Some(comuna.into()),
        creado_en: ahora,
    };
    vec![
        crear("Ada", "Lovelace", "ada@test.cl", "Apoderada", 13, "13101"),
        crear("Alan", "Turing", "alan@test.cl", "Apoderado", 5, "05101"),
        crear("Grace", "Hopper", "grace@test.cl", "Apoderada", 8, "08101"),
        crear("Linus", "Torvalds", "linus@test.cl", "Apoderado", 13, "13123"),
        crear("Marie", "Curie", "marie@test.cl", "Apoderada", 8, "08102"),
    ]
}

/// Tres alumnos y tres apoderados de ejemplo con RUT válidos.
pub fn datos_semilla_alumnos() -> Vec<Alumno> {
    let ahora = Utc::now();
    let crear = |rut: &str, nombres: &str, apellidos: &str, curso: &str| Alumno {
        id: Uuid::new_v4(),
        rut: rut.into(),
        nombres: nombres.into(),
        apellidos: apellidos.into(),
        fecha_nacimiento: None,
        curso: curso.into(),
        colegio_rbd: None,
        estado: EstadoAlumno::Postulante,
        creado_en: ahora,
    };
    vec![
        crear("12345678-5", "Martín", "González", "8° Básico A"),
        crear("11111111-1", "Sofía", "Fernández", "7° Básico B"),
        crear("6123456-K", "Diego", "Muñoz", "1° Medio A"),
    ]
}

/// Tres apoderados de ejemplo con RUT válidos.
pub fn datos_semilla_apoderados() -> Vec<Apoderado> {
    let ahora = Utc::now();
    let crear = |rut: &str, nombres: &str, apellidos: &str, region: i16, comuna: &str| Apoderado {
        id: Uuid::new_v4(),
        rut: rut.into(),
        nombres: nombres.into(),
        apellidos: apellidos.into(),
        telefono: "+56 9 5555 0100".into(),
        correo: "".into(),
        direccion: "".into(),
        region_codigo: Some(region),
        comuna_codigo: Some(comuna.into()),
        creado_en: ahora,
    };
    vec![
        crear("1-9", "Carmen", "González", 13, "13101"),
        crear("12345678-5", "Pedro", "Fernández", 5, "05101"),
        crear("11111111-1", "Ana", "Muñoz", 8, "08101"),
    ]
}
pub fn datos_semilla_oportunidades(ids_personas: &[Uuid]) -> Vec<Oportunidad> {
    let ahora = Utc::now();
    let contacto = |i: usize| ids_personas.get(i % ids_personas.len()).copied();
    let crear = |nombre: &str, importe: i64, etapa: EtapaOportunidad, i: usize| Oportunidad {
        id: Uuid::new_v4(),
        nombre: nombre.into(),
        importe,
        etapa,
        colegio_rbd: None,
        contacto_id: contacto(i),
        fecha_cierre: None,
        creado_en: ahora,
    };
    use EtapaOportunidad::*;
    vec![
        crear("Postulación — Informática 2026", 120_000, Contacto, 0),
        crear("Postulación — Ingeniería 2026", 85_000, Postulante, 1),
        crear("Postulación — Medicina 2026", 24_000, Evaluado, 2),
        crear("Postulación — Derecho 2026", 150_000, Evaluado, 3),
        crear("Postulación — Arquitectura 2026", 310_000, Aceptado, 4),
        crear("Matrícula — Industrial 2025", 480_000, Matriculado, 0),
    ]
}

/// Dos tareas y una nota de ejemplo para modo sin conexión.
pub fn datos_semilla_tareas() -> Vec<Tarea> {
    let ahora = Utc::now();
    let crear = |titulo: &str, estado: EstadoTarea| Tarea {
        id: Uuid::new_v4(),
        titulo: titulo.into(),
        detalle: String::new(),
        estado,
        prioridad: "media".into(),
        vencimiento: None,
        alumno_id: None,
        apoderado_id: None,
        oportunidad_id: None,
        creado_en: ahora,
    };
    vec![
        crear("Llamar apoderado", EstadoTarea::Pendiente),
        crear("Preparar entrevista", EstadoTarea::EnCurso),
    ]
}

/// Una nota de ejemplo para modo sin conexión.
pub fn datos_semilla_notas() -> Vec<Nota> {
    vec![Nota {
        id: Uuid::new_v4(),
        titulo: "Entrevista".into(),
        cuerpo: "Comprometido con el proceso.".into(),
        alumno_id: None,
        apoderado_id: None,
        oportunidad_id: None,
        creado_en: Utc::now(),
    }]
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::dominio::calcular_resumen;

    #[test]
    fn calcula_resumen_demo() {
        let colegios = datos_semilla_colegios();
        let personas = datos_semilla_personas();
        let ids_personas: Vec<Uuid> = personas.iter().map(|p| p.id).collect();
        let oportunidades = datos_semilla_oportunidades(&ids_personas);
        let resumen = calcular_resumen(&colegios, &personas, &oportunidades);
        assert_eq!(resumen.total_colegios, 4);
        assert_eq!(resumen.total_oportunidades, 6);
        assert!(resumen.valor_tuberia > 0);
        assert_eq!(resumen.valor_ganado, 480_000);
    }
}
