//! Importación CSV escolar con previsualización y aplicación por fila.

use axum::{Json, extract::{Multipart,Path,Query,State}, http::StatusCode};
use serde::Deserialize;
use sqlx::PgPool;

use crate::{auth::UsuarioActual,errores::ErrorApi};

#[derive(Debug,Deserialize)]
pub struct ParametrosImportacion { #[serde(default)] pub aplicar:bool }

pub async fn importar(
    State(pool):State<PgPool>,
    axum::Extension(usuario):axum::Extension<UsuarioActual>,
    Path(objeto):Path<String>,
    Query(params):Query<ParametrosImportacion>,
    mut multipart:Multipart,
)->Result<(StatusCode,Json<serde_json::Value>),ErrorApi>{
    if !matches!(objeto.as_str(),"alumnos"|"apoderados") {
        return Err(ErrorApi::Validacion("Solo se importan alumnos o apoderados.".into()));
    }
    let mut contenido=None;
    while let Some(campo)=multipart.next_field().await.map_err(|_|ErrorApi::Validacion("Formulario multipart no válido.".into()))? {
        if campo.name()==Some("archivo") {
            let bytes=campo.bytes().await.map_err(|_|ErrorApi::Validacion("No se pudo leer el archivo CSV.".into()))?;
            if bytes.len()>5*1024*1024 { return Err(ErrorApi::Validacion("El CSV no puede superar 5 MB.".into())); }
            contenido=Some(bytes);
            break;
        }
    }
    let contenido=contenido.ok_or_else(||ErrorApi::Validacion("Selecciona un archivo CSV.".into()))?;
    let mut reader=csv::ReaderBuilder::new().trim(csv::Trim::All).from_reader(contenido.as_ref());
    let headers=reader.headers().map_err(|_|ErrorApi::Validacion("La primera fila debe contener encabezados CSV.".into()))?.clone();
    let requeridos: &[&str] = &["rut", "nombres", "apellidos"];
    for h in requeridos {
        if !headers.iter().any(|x|x.eq_ignore_ascii_case(h)) {
            return Err(ErrorApi::Validacion(format!("Falta el encabezado obligatorio `{h}`.")));
        }
    }
    let indice=|nombre:&str|headers.iter().position(|h|h.eq_ignore_ascii_case(nombre));
    let mut filas=Vec::new();
    let mut previa=Vec::new();
    for (i,registro) in reader.records().enumerate(){
        if i>=1000 { return Err(ErrorApi::Validacion("El archivo no puede superar 1000 filas.".into())); }
        let registro=registro.map_err(|_|ErrorApi::Validacion(format!("La fila {} no tiene formato CSV válido.",i+2)))?;
        let get=|campo:&str|indice(campo).and_then(|j|registro.get(j)).unwrap_or("").trim().to_string();
        let fila_num=i+2;
        let mut datos=if objeto=="alumnos" {
            serde_json::json!({"fila":fila_num,"rut":get("rut"),"nombres":get("nombres"),"apellidos":get("apellidos"),"curso":get("curso"),"colegio_rbd":get("colegio_rbd"),"estado":if get("estado").is_empty(){"postulante".to_string()}else{get("estado")}})
        } else {
            serde_json::json!({"fila":fila_num,"rut":get("rut"),"nombres":get("nombres"),"apellidos":get("apellidos"),"telefono":get("telefono"),"correo":get("correo"),"direccion":get("direccion"),"region_codigo":get("region_codigo"),"comuna_codigo":get("comuna_codigo")})
        };
        let error=if objeto=="alumnos" {
            let estado_txt=datos["estado"].as_str().unwrap_or("postulante").to_lowercase();
            match occb_core::EstadoAlumno::desde_texto(&estado_txt) {
                None=>Some("Estado de alumno no válido.".to_string()),
                Some(estado)=>{
                    let entrada=occb_core::CrearAlumno{rut:datos["rut"].as_str().unwrap_or("").into(),nombres:datos["nombres"].as_str().unwrap_or("").into(),apellidos:datos["apellidos"].as_str().unwrap_or("").into(),fecha_nacimiento:None,curso:datos["curso"].as_str().unwrap_or("").into(),colegio_rbd:datos["colegio_rbd"].as_str().filter(|s|!s.is_empty()).map(str::to_string),estado};
                    occb_core::validar_alumno(entrada).err().map(|e|e.to_string())
                }
            }
        } else {
            let entrada=occb_core::CrearApoderado{rut:get_from(&datos,"rut"),nombres:get_from(&datos,"nombres"),apellidos:get_from(&datos,"apellidos"),telefono:get_from(&datos,"telefono"),correo:get_from(&datos,"correo"),direccion:get_from(&datos,"direccion"),region_codigo:get_from(&datos,"region_codigo").parse().ok(),comuna_codigo:Some(get_from(&datos,"comuna_codigo")).filter(|s|!s.is_empty())};
            occb_core::validar_apoderado(entrada).err().map(|e|e.to_string())
        };
        if i<5 {
            previa.push(serde_json::json!({"fila":fila_num,"rut":datos["rut"],"nombres":datos["nombres"],"error":error}));
        }
        if let Some(err)=&error { datos["error_local"]=serde_json::Value::String(err.clone()); }
        filas.push(datos);
    }
    if filas.is_empty(){return Err(ErrorApi::Validacion("El CSV no contiene filas de datos.".into()));}
    if !params.aplicar {
        return Ok((StatusCode::OK,Json(serde_json::json!({"objeto":objeto,"total":filas.len(),"previsualizacion":previa,"aplicado":false}))));
    }
    let funcion=if objeto=="alumnos" {"select importar_alumnos($1,$2)"} else {"select importar_apoderados($1,$2)"};
    let resultado:serde_json::Value=sqlx::query_scalar(funcion).bind(usuario.id).bind(serde_json::Value::Array(filas)).fetch_one(&pool).await?;
    Ok((StatusCode::OK,Json(serde_json::json!({"objeto":objeto,"aplicado":true,"previsualizacion":previa,"resultado":resultado}))))
}

fn get_from(v:&serde_json::Value,k:&str)->String{v.get(k).and_then(|x|x.as_str()).unwrap_or("").to_string()}
