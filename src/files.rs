//! Archivos en el navegador: leer el primero de un `<input type="file">`
//! y descargarlo. Fuera de wasm devuelven `Err`.

/// Lee el primer archivo del input con ese `id`.
/// Devuelve `(nombre, mime, bytes)`.
pub async fn read_first_file(element_id: &str) -> Result<(String, String, Vec<u8>), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().ok_or("Sin ventana de navegador")?;
        let document = window.document().ok_or("Sin documento")?;
        let input: web_sys::HtmlInputElement = document
            .get_element_by_id(element_id)
            .ok_or("No se encontró el selector de archivo")?
            .dyn_into()
            .map_err(|_| "Selector inválido")?;
        let files = input.files().ok_or("Sin archivos elegidos")?;
        let file = files.get(0).ok_or("Elige un archivo primero")?;
        let name = file.name();
        let mime = file.type_();
        let promise = file.array_buffer();
        let buffer =
            wasm_bindgen_futures::JsFuture::from(promise)
                .await
                .map_err(|_| "No se pudo leer el archivo")?;
        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        Ok((name, mime, bytes))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = element_id;
        Err("Lectura de archivos disponible solo en el navegador".to_string())
    }
}
