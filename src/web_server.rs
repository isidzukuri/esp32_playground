use embedded_svc::http::Method;
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::io::EspIOError;
use std::str;
use std::sync::{mpsc::channel, Arc, Mutex};
use std::thread;
use storage::StorageClassTrait;
use storage::StorageControllerTrait;

#[cfg(esp_idf_httpd_ws_support)]
use embedded_svc::ws::FrameType;
#[cfg(esp_idf_httpd_ws_support)]
use esp_idf_svc::sys::EspError;
#[cfg(esp_idf_httpd_ws_support)]
use std::collections::BTreeMap;

const INDEX_HTML: &str = include_str!("assets/index.html");
const SCRIPTS_JS: &str = include_str!("assets/scripts.js");
const STYLES_CSS: &str = include_str!("assets/styles.css");
const WS_MSG_MAX_LEN: usize = 1024;
const MAX_OPEN_SOCKETS: usize = 4; // must match or be lower than CONFIG_LWIP_MAX_SOCKETS
const MAX_SESSIONS: usize = 7;

pub fn start_web_server<StorageClass, Controller>(
    storage_controller: Arc<Mutex<Controller>>,
) -> std::result::Result<Arc<EspHttpServer<'static>>, EspIOError>
where
    StorageClass: StorageClassTrait + Default + Send + 'static,
    Controller: StorageControllerTrait<StorageClass> + std::marker::Sync + Send + 'static, // Ensure Controller is 'static and Send
{
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
    let locked_storage_controller = storage_controller.clone();
    server.fn_handler(
        "/data",
        Method::Get,
        move |request| -> Result<(), EspIOError> {
            let mut resp =
                request.into_response(200, Some("OK"), &[("Content-Type", "text/csv")])?;

            println!("[WebServer] -> /data");

            let closure_storage_controller = locked_storage_controller.lock().unwrap();

            closure_storage_controller
                .read_whole_storage(|reader| {
                    stream_to_response(reader, &mut resp);
                })
                .expect("[WebServer] storage controller closure error");
            println!("[WebServer] -> /data - completion");

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

pub fn stream_to_response<R, C>(mut reader: R, resp: &mut esp_idf_svc::http::server::Response<C>)
where
    R: std::io::Read,
    C: esp_idf_svc::http::server::Connection,
{
    // println!("[WebServer] -> stream_to_response");
    let mut buf = [0u8; 1024];
    loop {
        let itr = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(itr) => itr,
            Err(_) => return,
        };
        // println!("[WebServer] -> stream_to_response -> loop: {}", &itr);
        resp.write(&buf[..itr])
            .expect("[WebServer] stream_to_response write failure.");
    }
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
