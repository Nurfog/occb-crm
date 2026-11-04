//! Datos de demostración en español con relaciones coherentes.
//! Se usan en el frontal (modo sin conexión) y como referencia de `sembrar_datos_demo()` en SQL.

use super::dominio::{Colegio, EtapaOportunidad, Oportunidad, Persona};
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

/// Seis postulaciones repartidas por el embudo de admisión.
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
