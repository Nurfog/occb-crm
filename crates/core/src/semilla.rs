//! Datos de demostración en español con relaciones coherentes.
//! Se usan en el frontal (modo sin conexión) y como referencia de `sembrar_datos_demo()` en SQL.

use super::dominio::{Empresa, EtapaOportunidad, Oportunidad, Persona};
use chrono::Utc;
use uuid::Uuid;

/// Seis empresas de ejemplo ordenadas por nombre.
pub fn datos_semilla_empresas() -> Vec<Empresa> {
    let ahora = Utc::now();
    let crear = |nombre: &str, dominio: &str, sector: &str, empleados: i32, ciudad: &str| Empresa {
        id: Uuid::new_v4(),
        nombre: nombre.into(),
        dominio: dominio.into(),
        sector: sector.into(),
        empleados,
        ciudad: ciudad.into(),
        creado_en: ahora,
    };
    let mut lista = vec![
        crear("Acme Corp", "acme.com", "Software", 240, "Madrid"),
        crear("Globex", "globex.io", "Finanzas", 85, "Barcelona"),
        crear("Initech", "initech.dev", "Herramientas", 42, "Valencia"),
        crear("Umbrella", "umbrella.salud", "Salud", 510, "Bilbao"),
        crear("Hooli", "hooli.com", "Nube", 1200, "Sevilla"),
        crear("Stark Industries", "stark.ind", "Industria", 3000, "Zaragoza"),
    ];
    lista.sort_by(|a, b| a.nombre.cmp(&b.nombre));
    lista
}

/// Cinco personas de ejemplo; la relación con empresas se asigna fuera.
pub fn datos_semilla_personas() -> Vec<Persona> {
    let ahora = Utc::now();
    let crear = |nombre: &str, apellido: &str, correo: &str, puesto: &str, ciudad: &str| Persona {
        id: Uuid::new_v4(),
        nombre: nombre.into(),
        apellido: apellido.into(),
        correo: correo.into(),
        telefono: "+34 555 0100".into(),
        puesto: puesto.into(),
        empresa_id: None,
        ciudad: ciudad.into(),
        creado_en: ahora,
    };
    vec![
        crear("Ada", "Lovelace", "ada@acme.com", "Directora técnica", "Madrid"),
        crear("Alan", "Turing", "alan@globex.io", "Responsable de datos", "Barcelona"),
        crear("Grace", "Hopper", "grace@initech.dev", "VP Ingeniería", "Valencia"),
        crear("Linus", "Torvalds", "linus@hooli.com", "Ingeniero sénior", "Sevilla"),
        crear("Marie", "Curie", "marie@umbrella.salud", "Jefa de investigación", "Bilbao"),
    ]
}

/// Seis oportunidades repartidas por el embudo.
pub fn datos_semilla_oportunidades(
    ids_empresas: &[Uuid],
    ids_personas: &[Uuid],
) -> Vec<Oportunidad> {
    let ahora = Utc::now();
    let empresa = |i: usize| ids_empresas.get(i % ids_empresas.len()).copied();
    let contacto = |i: usize| ids_personas.get(i % ids_personas.len()).copied();
    let crear =
        |nombre: &str, importe: i64, etapa: EtapaOportunidad, i: usize| Oportunidad {
            id: Uuid::new_v4(),
            nombre: nombre.into(),
            importe,
            etapa,
            empresa_id: empresa(i),
            contacto_id: contacto(i),
            fecha_cierre: None,
            creado_en: ahora,
        };
    use EtapaOportunidad::*;
    vec![
        crear("Acme — Plan empresarial", 120_000, Nueva, 0),
        crear("Globex — Despliegue de pagos", 85_000, Contacto, 1),
        crear("Initech — Ampliación de licencias", 24_000, Reunion, 2),
        crear("Hooli — Migración a la nube", 310_000, Propuesta, 4),
        crear("Umbrella — Suite de investigación", 150_000, Reunion, 3),
        crear("Stark — IoT para fábricas", 480_000, Cliente, 5),
    ]
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::dominio::calcular_resumen;

    #[test]
    fn calcula_resumen_demo() {
        let empresas = datos_semilla_empresas();
        let ids_empresas: Vec<Uuid> = empresas.iter().map(|e| e.id).collect();
        let mut personas = datos_semilla_personas();
        for (i, p) in personas.iter_mut().enumerate() {
            p.empresa_id = ids_empresas.get(i % ids_empresas.len()).copied();
        }
        let ids_personas: Vec<Uuid> = personas.iter().map(|p| p.id).collect();
        let oportunidades = datos_semilla_oportunidades(&ids_empresas, &ids_personas);
        let resumen = calcular_resumen(&empresas, &personas, &oportunidades);
        assert_eq!(resumen.total_empresas, 6);
        assert_eq!(resumen.total_oportunidades, 6);
        assert!(resumen.valor_tuberia > 0);
        assert_eq!(resumen.valor_ganado, 480_000);
    }
}
