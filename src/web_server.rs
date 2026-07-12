use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use embedded_svc::http::Method;
use esp_idf_svc::io::EspIOError;
use std::io::Read;
use std::sync::{Arc, mpsc::{channel, Sender}};
use std::thread;
use std::str;

#[cfg(esp_idf_httpd_ws_support)]
use esp_idf_svc::sys::{EspError, ESP_ERR_INVALID_SIZE};
#[cfg(esp_idf_httpd_ws_support)]
use embedded_svc::ws::FrameType;
#[cfg(esp_idf_httpd_ws_support)]
use std::sync::Mutex;
#[cfg(esp_idf_httpd_ws_support)]
use std::collections::BTreeMap;

const INDEX_HTML: &str = include_str!("assets/index.html");
const SCRIPTS_JS: &str = include_str!("assets/scripts.js");
const STYLES_CSS: &str = include_str!("assets/styles.css");

pub fn start_web_server(log_path: &'static str) -> std::result::Result<Arc<EspHttpServer<'static>>, EspIOError> {
    let mut config = Configuration::default();
    config.max_open_sockets = 4; // must match or be lower than CONFIG_LWIP_MAX_SOCKETS
    config.max_sessions = 7;
    let mut server = EspHttpServer::new(&config)?;

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

    // shared map of detached WS senders (keyed by session fd)
    #[cfg(esp_idf_httpd_ws_support)]
    let ws_sessions: Arc<Mutex<BTreeMap<i32, esp_idf_svc::http::server::ws::EspHttpWsDetachedSender>>> =
    Arc::new(Mutex::new(BTreeMap::new()));

    #[cfg(esp_idf_httpd_ws_support)]
    let (ws_tx, ws_rx) = channel::<String>();

    #[cfg(esp_idf_httpd_ws_support)]
    {
      let ws_sessions = ws_sessions.clone();
      thread::spawn(move || {
          for broadcast in ws_rx {
              let mut sessions = ws_sessions.lock().unwrap();
              println!("WS background broadcasting to {} sessions", sessions.len());
              sessions.retain(|session, sender| {
                  match sender.send(FrameType::Text(false), broadcast.as_bytes()) {
                      Ok(()) => {
                          println!("WS background send success for session {}", session);
                          true
                      }
                      Err(err) => {
                          println!("WS background send failed for session {}: {:?}", session, err);
                          false
                      }
                  }
              });
          }
          println!("WS background thread exiting");
      });
    }

    // WebSocket chat endpoint: accepts frames with parameters `user_name` and `message`.
    // Incoming payload formats supported (in order): JSON with keys, query-string `user_name=..&message=..`,
    // or plain `user|message` or `user:message`. Broadcasts new messages to all connected clients.
    #[cfg(esp_idf_httpd_ws_support)]
    {
      let ws_sessions = ws_sessions.clone();
      let ws_tx = ws_tx.clone();
      server.ws_handler("/ws", None, move |connection| -> Result<(), EspError> {
        // Use EspError for WS handler errors
        let mut connection = connection;
        println!("WS handler entry is_new={} is_closed={}", connection.is_new(), connection.is_closed());
        // New connection: create detached sender and store it
        if connection.is_new() {

          let sender = connection.create_detached_sender()?;
          let fd = sender.session();
          let mut sessions = ws_sessions.lock().unwrap();
          sessions.insert(fd, sender);
          println!("WS new session added {}", fd);
          return Ok(());
        }

        // Closed connection: remove from sessions
        if connection.is_closed() {
          let session = connection.session();
          let mut sessions = ws_sessions.lock().unwrap();
          sessions.remove(&session);
          println!("WS session closed {}", session);
          return Ok(());
        }

        let session = connection.session();
        {
            let mut sessions = ws_sessions.lock().unwrap();
            if !sessions.contains_key(&session) {
                let sender = connection.create_detached_sender()?;
                sessions.insert(session, sender);
                println!("WS recovered detached sender for existing session {}", session);
            }
        }

        // Receiving a frame: first call with empty buffer to get length
        let (frame_type, len) = match connection.recv(&mut []) {
            Ok(v) => v,
            Err(err) => {
                println!("WS recv header failed: {:?}", err);
                return Err(err);
            }
        };
        println!("WS recv header: {:?} len={}", frame_type, len);

        if !matches!(frame_type, FrameType::Text(_) | FrameType::Binary(_)) {
          println!("WS ignored non-data frame: {:?}", frame_type);
          return Ok(());
        }

        const MAX_LEN: usize = 1024;
        if len > MAX_LEN {
          // consume the oversized frame before leaving the handler
          let mut discard = vec![0u8; len];
          connection.recv(discard.as_mut_slice())?;
          connection.send(FrameType::Text(false), b"Request too big")?;
          connection.send(FrameType::Close, &[])?;
          return Err(EspError::from_infallible::<ESP_ERR_INVALID_SIZE>());
        }

        let mut buf = [0u8; MAX_LEN];
        let (_frame_type2, len2) = connection.recv(buf.as_mut())?;
        println!("WS recv payload: len2={}", len2);

        let actual_len = if len2 > 0 && buf[len2 - 1] == 0 { len2 - 1 } else { len2 };

        let text = match str::from_utf8(&buf[..actual_len]) {
          Ok(s) => s,
          Err(err) => {
            println!("WS utf8 error: {:?}", err);
            return Ok(());
          }
        };

        // parse simple payloads to extract user_name and message
        let (user_name, message) = if text.contains("{") && text.contains("user_name") {

println!("WS parse simple payloads");

          // crude JSON extraction to avoid adding serde dependency
          let uname = text
            .split("\"user_name\"")
            .nth(1)
            .and_then(|s| s.split(':').nth(1))
            .and_then(|s| s.split('"').nth(1))
            .unwrap_or("");
          let msg = text
            .split("\"message\"")
            .nth(1)
            .and_then(|s| s.split(':').nth(1))
            .and_then(|s| s.split('"').nth(1))
            .unwrap_or("");
          (uname.to_string(), msg.to_string())
        } else if text.contains("user_name=") {
          let mut uname = "";
          let mut msg = "";
          for part in text.split('&') {
            if let Some(v) = part.strip_prefix("user_name=") { uname = v; }
            if let Some(v) = part.strip_prefix("message=") { msg = v; }
          }
          (uname.to_string(), msg.to_string())
        } else if let Some(pos) = text.find('|') {
          (text[..pos].to_string(), text[pos+1..].to_string())
        } else if let Some(pos) = text.find(':') {
          (text[..pos].to_string(), text[pos+1..].to_string())
        } else {
          ("".to_string(), text.to_string())
        };

        // build broadcast payload (simple JSON)
        let broadcast = format!(
          "{{\"user_name\":\"{}\",\"message\":\"{}\"}}",
          user_name.replace('"', "'"),
          message.replace('"', "'")
        );

        
        
        // queue broadcast for background delivery
        println!("WS queueing broadcast: {}", broadcast);
        if let Err(err) = ws_tx.send(broadcast) {
            println!("WS broadcast channel send failed: {:?}", err);
        }

        println!("WS sent.");

        Ok(())
      })?;
    }

    

    Ok(Arc::new(server))
}
