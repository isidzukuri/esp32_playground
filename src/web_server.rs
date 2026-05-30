use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use embedded_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use std::sync::Arc;

const INDEX_HTML: &str = r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <title>Sound Monitor</title>
  <link rel="stylesheet" href="/styles.css">
</head>
<body>
  <h1>Sound Monitor</h1>
  <canvas id="plot" width="800" height="200"></canvas>
  <script src="/scripts.js"></script>
</body>
</html>"#;

const SCRIPTS_JS: &str = r#"
// Simple client: fetch CSV and log length
async function fetchCsv() {
  const r = await fetch('/data');
  const reader = r.body.getReader();
  const decoder = new TextDecoder();
  let csv = '';
  for (;;) {
    const { value, done } = await reader.read();
    if (done) break;
    csv += decoder.decode(value, { stream: true });
  }
  console.log('CSV length', csv.length);
}
fetchCsv();
"#;

const STYLES_CSS: &str = r#"
body { font-family: sans-serif; margin: 12px; }
canvas { border: 1px solid #ccc; display:block; margin-top:12px; }
"#;

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
