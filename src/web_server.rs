use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use embedded_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use std::sync::Arc;

const INDEX_HTML: &str = include_str!("assets/index.html");
const SCRIPTS_JS: &str = include_str!("assets/scripts.js");
const STYLES_CSS: &str = include_str!("assets/styles.css");

pub fn start_web_server(log_path: &'static str) -> std::result::Result<Arc<EspHttpServer<'static>>, EspIOError> {
    let mut server = EspHttpServer::new(&Configuration::default())?;

    // index
    server.fn_handler("/", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_ok_response()?;
      let _ = resp.write(INDEX_HTML.as_bytes())?;
      Ok(())
    })?;

    // scripts.js
    server.fn_handler("/scripts.js", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_ok_response()?;
      let _ = resp.write(SCRIPTS_JS.as_bytes())?;
      Ok(())
    })?;

    // styles.css
    server.fn_handler("/styles.css", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_ok_response()?;
      let _ = resp.write(STYLES_CSS.as_bytes())?;
      Ok(())
    })?;

    // stream CSV (read whole file and write; if missing, return empty)
    let path = log_path.to_owned();
    server.fn_handler("/data", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_ok_response()?;
      // best-effort: read file and write once
      match std::fs::read(&path) {
        Ok(buf) => {
          let _ = resp.write(&buf)?;
          Ok(())
        }
        Err(_) => Ok(()),
      }
    })?;

    Ok(Arc::new(server))
}
