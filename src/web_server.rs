use embedded_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::io::Read;
use std::str;
use std::sync::{mpsc::channel, Arc};
use std::thread;

#[cfg(esp_idf_httpd_ws_support)]
use embedded_svc::ws::FrameType;
#[cfg(esp_idf_httpd_ws_support)]
use esp_idf_svc::sys::EspError;
#[cfg(esp_idf_httpd_ws_support)]
use std::collections::BTreeMap;
#[cfg(esp_idf_httpd_ws_support)]
use std::sync::Mutex;

const INDEX_HTML: &str = include_str!("assets/index.html");
const SCRIPTS_JS: &str = include_str!("assets/scripts.js");
const STYLES_CSS: &str = include_str!("assets/styles.css");
const WS_MSG_MAX_LEN: usize = 1024;
const MAX_OPEN_SOCKETS: usize = 4; // must match or be lower than CONFIG_LWIP_MAX_SOCKETS
const MAX_SESSIONS: usize = 7;

pub fn start_web_server(
    log_path: &'static str,
) -> std::result::Result<Arc<EspHttpServer<'static>>, EspIOError> {
    let config = Configuration {
        max_open_sockets: MAX_OPEN_SOCKETS,
        max_sessions: MAX_SESSIONS,
        ..Default::default()
    };
    let mut server = EspHttpServer::new(&config)?;

    // index
    server.fn_handler("/", Method::Get, move |request| -> Result<(), EspIOError> {
        let mut resp = request.into_ok_response()?;
        let _ = resp.write(INDEX_HTML.as_bytes())?;
        Ok(())
    })?;

    // scripts.js
    server.fn_handler(
        "/scripts.js",
        Method::Get,
        move |request| -> Result<(), EspIOError> {
            let mut resp =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/javascript")])?;
            let _ = resp.write(SCRIPTS_JS.as_bytes())?;
            Ok(())
        },
    )?;

    // styles.css
    server.fn_handler(
        "/styles.css",
        Method::Get,
        move |request| -> Result<(), EspIOError> {
            let mut resp =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/css")])?;
            let _ = resp.write(STYLES_CSS.as_bytes())?;
            Ok(())
        },
    )?;

    // stream CSV (read in chunks so large files don't fill RAM)
    let path = log_path.to_owned();
    server.fn_handler(
        "/data",
        Method::Get,
        move |request| -> Result<(), EspIOError> {
            let mut resp =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/csv")])?;

            let mut file = match std::fs::File::open(&path) {
                Ok(file) => file,
                Err(_) => return Ok(()),
            };

            let mut buf = [0u8; 1024];
            loop {
                let itr = match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(itr) => itr,
                    Err(_) => return Ok(()),
                };

                resp.write(&buf[..itr])?;
            }

            Ok(())
        },
    )?;

    // TODO: refactor WS code.
    #[cfg(esp_idf_httpd_ws_support)]
    {
        let (ws_tx, ws_rx) = channel::<String>();
        // shared map of detached WS senders (keyed by session fd)
        let ws_sessions: Arc<
            Mutex<BTreeMap<i32, esp_idf_svc::http::server::ws::EspHttpWsDetachedSender>>,
        > = Arc::new(Mutex::new(BTreeMap::new()));

        spawn_active_sessions_threds(ws_sessions.clone(), ws_rx);

        // WebSocket chat endpoint: accepts frames with parameters `user_name` and `message`.
        // Incoming payload formats supported (in order): JSON with keys, query-string `user_name=..&message=..`,
        // or plain `user|message` or `user:message`. Broadcasts new messages to all connected clients.
        let ws_sessions = ws_sessions.clone();
        server.ws_handler("/ws", None, move |connection| -> Result<(), EspError> {
            // Use EspError for WS handler errors
            // New connection: create detached sender and store it
            if connection.is_new() {
                let sender = connection.create_detached_sender()?;
                let fd = sender.session();
                let mut sessions = ws_sessions
                    .lock()
                    .expect("Failed to lock WebSocket sessions");
                sessions.insert(fd, sender);
                return Ok(());
            }

            // Closed connection: remove from sessions
            if connection.is_closed() {
                let session = connection.session();
                let mut sessions = ws_sessions
                    .lock()
                    .expect("Failed to lock WebSocket sessions");
                sessions.remove(&session);
                return Ok(());
            }

            let _ = handle_ws_incoming_message(connection, ws_tx.clone());

            Ok(())
        })?;
    }

    Ok(Arc::new(server))
}

#[cfg(esp_idf_httpd_ws_support)]
fn spawn_active_sessions_threds(
    ws_sessions: Arc<Mutex<BTreeMap<i32, esp_idf_svc::http::server::ws::EspHttpWsDetachedSender>>>,
    ws_rx: std::sync::mpsc::Receiver<std::string::String>,
) {
    thread::spawn(move || {
        for broadcast in ws_rx {
            let mut sessions = ws_sessions
                .lock()
                .expect("Failed to lock WebSocket sessions");
            sessions.retain(|_, sender| {
                sender
                    .send(FrameType::Text(false), broadcast.as_bytes())
                    .is_ok()
            });
        }
    });
}

#[cfg(esp_idf_httpd_ws_support)]
fn handle_ws_incoming_message(
    connection: &mut esp_idf_svc::http::server::ws::EspHttpWsConnection,
    ws_tx: std::sync::mpsc::Sender<std::string::String>,
) -> Result<(), EspIOError> {
    // Receiving a frame: first call with empty buffer to get length
    let (_frame_type, len) = connection.recv(&mut [])?;
    if len > WS_MSG_MAX_LEN {
        // ignore too large messages
        return Ok(());
    }

    let mut buf = [0u8; WS_MSG_MAX_LEN];
    connection.recv(buf.as_mut())?;

    let text = match str::from_utf8(&buf[..len]) {
        Ok(st) => st,
        Err(_) => return Ok(()),
    };
    // parse simple payloads to extract user_name and message
    let (user_name, message) = parse_ws_message(text);
    // build broadcast payload (simple JSON)
    let broadcast = format!(
        "{{\"user_name\":\"{}\",\"message\":\"{}\"}}",
        user_name.replace('"', "'"),
        message.replace('"', "'")
    );
    // queue broadcast for background delivery
    let _ = ws_tx.send(broadcast);

    Ok(())
}

// crude JSON extraction to avoid adding serde dependency
#[cfg(esp_idf_httpd_ws_support)]
fn parse_ws_message(text: &str) -> (String, String) {
    if text.contains("{") && text.contains("user_name") {
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
            if let Some(val) = part.strip_prefix("user_name=") {
                uname = val;
            }
            if let Some(val) = part.strip_prefix("message=") {
                msg = val;
            }
        }
        (uname.to_string(), msg.to_string())
    } else if let Some(pos) = text.find('|') {
        (text[..pos].to_string(), text[pos + 1..].to_string())
    } else if let Some(pos) = text.find(':') {
        (text[..pos].to_string(), text[pos + 1..].to_string())
    } else {
        ("".to_string(), text.to_string())
    }
}

// TODO: fix error, after some period in terminal appears
//  (1162931) httpd_ws: httpd_ws_recv_frame: WS frame is not properly masked.
// W (1162941) httpd_txrx: httpd_sock_err: error in recv : 128
// W (1162941) httpd_txrx: httpd_sock_err: error in recv : 128
