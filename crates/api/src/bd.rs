//! Capa de acceso a datos: solo llama a funciones de PostgreSQL.
//! La API jamás hace INSERT/UPDATE/DELETE directos (buena práctica de seguridad).

use sqlx::{PgPool, Row, postgres::PgPoolOptions};

/// Crea el pool de conexiones a PostgreSQL.
pub async fn crear_pool(url_bd: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(10)
        .connect(url_bd)
        .await
}

/// Ejecuta las migraciones empaquetadas en `migrations/` y siembra demo si está vacío.
pub async fn migrar_y_sembrar(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    // Siembra idempotente (la función no hace nada si ya hay empresas).
    sqlx::query("select sembrar_datos_demo()")
        .execute(pool)
        .await?;
    Ok(())
}

/// Fila de empresa leída desde una función.
pub fn fila_a_empresa(fila: &sqlx::postgres::PgRow) -> occb_core::Empresa {
    occb_core::Empresa {
        id: fila.get("id"),
        nombre: fila.get("nombre"),
        dominio: fila.get::<String, _>("dominio"),
        sector: fila.get::<String, _>("sector"),
        empleados: fila.get::<i32, _>("empleados"),
        ciudad: fila.get::<String, _>("ciudad"),
        creado_en: fila.get("creado_en"),
    }
}

/// Fila de persona leída desde una función.
pub fn fila_a_persona(fila: &sqlx::postgres::PgRow) -> occb_core::Persona {
    occb_core::Persona {
        id: fila.get("id"),
        nombre: fila.get("nombre"),
        apellido: fila.get("apellido"),
        correo: fila.get("correo"),
        telefono: fila.get::<String, _>("telefono"),
        puesto: fila.get::<String, _>("puesto"),
        empresa_id: fila.get::<Option<uuid::Uuid>, _>("empresa_id"),
        ciudad: fila.get::<String, _>("ciudad"),
        creado_en: fila.get("creado_en"),
    }
}

/// Fila de oportunidad; la etapa llega como texto y se convierte al enum.
pub fn fila_a_oportunidad(fila: &sqlx::postgres::PgRow) -> occb_core::Oportunidad {
    let etapa_texto: String = fila.get("etapa");
    occb_core::Oportunidad {
        id: fila.get("id"),
        nombre: fila.get("nombre"),
        importe: fila.get::<i64, _>("importe"),
        etapa: occb_core::EtapaOportunidad::desde_texto(&etapa_texto)
            .unwrap_or(occb_core::EtapaOportunidad::Nueva),
        empresa_id: fila.get::<Option<uuid::Uuid>, _>("empresa_id"),
        contacto_id: fila.get::<Option<uuid::Uuid>, _>("contacto_id"),
        fecha_cierre: fila.get::<Option<chrono::DateTime<chrono::Utc>>, _>("fecha_cierre"),
        creado_en: fila.get("creado_en"),
    }
}
