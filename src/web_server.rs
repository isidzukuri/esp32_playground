use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use embedded_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use std::io::Read;
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
      let mut resp = request.into_response(
          200, 
          Some("OK"), 
          &[("Content-Type", "text/javascript")]
      )?;
      let _ = resp.write(SCRIPTS_JS.as_bytes())?;
      Ok(())
    })?;

    // styles.css
    server.fn_handler("/styles.css", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_response(
          200, 
          Some("OK"), 
          &[("Content-Type", "text/css")]
      )?;
      let _ = resp.write(STYLES_CSS.as_bytes())?;
      Ok(())
    })?;

    // stream CSV (read in chunks so large files don't fill RAM)
    let path = log_path.to_owned();
    server.fn_handler("/data", Method::Get, move |request| -> Result<(), EspIOError> {
      let mut resp = request.into_response(
          200,
          Some("OK"),
          &[("Content-Type", "text/csv")],
      )?;

      let mut file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(_) => return Ok(()),
      };

      let mut buf = [0u8; 1024];
      loop {
        let n = match file.read(&mut buf) {
          Ok(0) => break,
          Ok(n) => n,
          Err(_) => return Ok(()),
        };

        resp.write(&buf[..n])?;
      }

      Ok(())
    })?;

    Ok(Arc::new(server))
}
