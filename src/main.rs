mod session_manager;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Json},
    routing::{delete, get, post},
    Router,
};
use serde::Deserialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

#[derive(Deserialize)]
struct MessagesQuery {
    source: String,
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct ResumeQuery {
    command: String,
    cwd: Option<String>,
}

#[derive(Deserialize)]
struct DeleteQuery {
    source: String,
    session_id: String,
}

#[derive(serde::Serialize)]
struct DeleteResult {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

async fn list_sessions() -> Json<Vec<session_manager::SessionMeta>> {
    Json(session_manager::scan_sessions())
}

async fn get_messages(
    Query(query): Query<MessagesQuery>,
) -> Result<Json<session_manager::PaginatedMessages>, (StatusCode, Json<serde_json::Value>)> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    match session_manager::load_messages_paginated(&query.source, offset, limit) {
        Ok(result) => Ok(Json(result)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )),
    }
}

async fn resume_session(
    Query(query): Query<ResumeQuery>,
) -> Json<DeleteResult> {
    match launch_terminal(&query.command, query.cwd.as_deref()) {
        Ok(_) => Json(DeleteResult { success: true, error: None }),
        Err(e) => Json(DeleteResult { success: false, error: Some(e) }),
    }
}

/// Build a list of `Command` candidates to try, one per available terminal on this OS.
fn terminal_candidates(command: &str, cwd: &str) -> Vec<std::process::Command> {
    let mut cmds: Vec<std::process::Command> = Vec::new();

    // macOS: AppleScript via osascript
    #[cfg(target_os = "macos")]
    {
        let esc = cwd.replace('\'', "'\\''");
        for (_, template) in [
            ("Terminal.app", "tell app \"Terminal\" to do script \"cd '{cwd}' && {cmd}\""),
            ("iTerm", "tell app \"iTerm\" to tell current window to create tab with default profile command \"cd '{cwd}' && {cmd}\""),
        ] {
            let script = template.replace("{cwd}", &esc).replace("{cmd}", command);
            let mut c = std::process::Command::new("osascript");
            c.args(["-e", &script]);
            cmds.push(c);
        }
    }

    // Windows: cmd or wt
    #[cfg(target_os = "windows")]
    {
        let mut c1 = std::process::Command::new("cmd");
        c1.args(["/c", "start", "cmd", "/k", &format!("cd /d \"{cwd}\" && {command}")]);
        cmds.push(c1);

        let mut c2 = std::process::Command::new("wt");
        c2.args(["-d", cwd, "cmd", "/k", command]);
        cmds.push(c2);
    }

    // Linux: try various terminal emulators
    #[cfg(target_os = "linux")]
    {
        let shell_cmd = format!("cd \"{cwd}\" && {command}; exec $SHELL");
        for (bin, args) in [
            ("gnome-terminal", &[][..]),
            ("ghostty", &["-e", "bash", "-c"][..]),
            ("konsole", &["-e", "bash", "-c"]),
            ("xfce4-terminal", &["-e", "bash", "-c"]),
            ("kitty", &["bash", "-c"]),
            ("alacritty", &["-e", "bash", "-c"]),
            ("wezterm", &["start", "--", "bash", "-c"]),
            ("x-terminal-emulator", &["-e", "bash", "-c"]),
            ("xterm", &["-e", "bash", "-c"]),
        ] {
            let mut c = std::process::Command::new(bin);
            if args.is_empty() {
                // gnome-terminal: -- bash -c <cmd>
                c.args(["--", "bash", "-c", &shell_cmd]);
            } else {
                c.args(args).arg(&shell_cmd);
            }
            cmds.push(c);
        }
    }

    cmds
}

/// Try each terminal candidate; return on first success.
fn launch_terminal(command: &str, cwd: Option<&str>) -> Result<(), String> {
    let cwd = cwd.filter(|p| !p.is_empty()).unwrap_or(".");

    for mut cmd in terminal_candidates(command, cwd) {
        cmd.stdin(std::process::Stdio::null())
           .stdout(std::process::Stdio::null())
           .stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(_) => return Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => continue,
        }
    }

    Err(format!("no terminal found; run manually: {command}"))
}

async fn delete_session_handler(
    Query(query): Query<DeleteQuery>,
) -> Json<DeleteResult> {
    match session_manager::delete_session(&query.session_id, &query.source) {
        Ok(_) => Json(DeleteResult { success: true, error: None }),
        Err(e) => Json(DeleteResult { success: false, error: Some(e) }),
    }
}

async fn serve_index() -> Html<&'static str> {
    Html(include_str!("static/index.html"))
}

async fn shutdown(State(flag): State<Arc<AtomicBool>>) -> Json<DeleteResult> {
    flag.store(true, Ordering::SeqCst);
    Json(DeleteResult { success: true, error: None })
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let app = Router::new()
        .route("/api/sessions", get(list_sessions))
        .route("/api/sessions/messages", get(get_messages))
        .route("/api/sessions", delete(delete_session_handler))
        .route("/api/resume", post(resume_session))
        .route("/api/shutdown", post(shutdown))
        .fallback(get(serve_index))
        .with_state(shutdown_flag.clone())
        .layer(cors);

    let addr = "0.0.0.0:8888";
    println!("Session Manager  http://localhost:8888");

    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg("http://localhost:8888")
            .spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open")
            .arg("http://localhost:8888")
            .spawn();
    }

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server = axum::serve(listener, app);

    tokio::select! {
        _ = server => {},
        _ = async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                if shutdown_flag.load(Ordering::SeqCst) {
                    break;
                }
            }
        } => {},
    }

    println!("Server stopped.");
    std::process::exit(0);
}
