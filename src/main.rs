use flexi_logger::{FileSpec, Logger};
use log;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const REFRESH_EVENTS: &[&str] = &[
    "activewindow",
    "activewindowv2",
    "closewindow",
    "workspace",
    "workspacev2",
    "movewindow",
    "movewindowv2",
    "windowtitle",
    "windowtitlev2",
];

const HELP: &str = "\
hyprland-activewindow: a multi monitor aware active hyprland window title reporter, designed to be used with eww.

USAGE:
  hyprland-activewindow MONITOR

FLAGS:
  -h, --help            Prints help information

ARGS:
  <MONITOR>             Monitor to report active window title on or _ to report on all monitors
                        Note: using _ will output in json format
";

#[derive(Serialize)]
struct MonitorCustom {
    pub name: String,
    pub title: String,
    pub initial_title: String,
}

#[derive(Deserialize)]
struct WorkspaceRef {
    id: i32,
}

#[derive(Deserialize)]
struct Monitor {
    name: String,
    #[serde(rename = "activeWorkspace")]
    active_workspace: WorkspaceRef,
}

#[derive(Deserialize)]
struct Workspace {
    id: i32,
    #[serde(rename = "lastwindow")]
    last_window: String,
    #[serde(rename = "lastwindowtitle")]
    last_window_title: String,
}

#[derive(Deserialize)]
struct Client {
    address: String,
    title: String,
    #[serde(rename = "initialTitle")]
    initial_title: String,
}

fn socket_path(socket: &str) -> Result<PathBuf> {
    let sig = env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map_err(|_| "HYPRLAND_INSTANCE_SIGNATURE not set, is Hyprland running?")?;
    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        let path = PathBuf::from(runtime_dir).join("hypr").join(&sig).join(socket);
        if path.exists() {
            return Ok(path);
        }
    }
    Ok(PathBuf::from("/tmp/hypr").join(sig).join(socket))
}

fn query<T: DeserializeOwned>(cmd: &str) -> Result<T> {
    let mut stream = UnixStream::connect(socket_path(".socket.sock")?)?;
    stream.write_all(format!("j/{cmd}").as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(serde_json::from_str(&response)?)
}

fn output(mon: &str, last: &mut Option<String>) {
    let current = if mon == "_" {
        get_all().unwrap_or_else(|err| {
            log::error!("Error getting all: {}", err);
            String::new()
        })
    } else {
        get_single(mon).unwrap_or_else(|err| {
            log::error!("Error getting single: {}", err);
            String::new()
        })
    };
    if last.as_ref() != Some(&current) {
        println!("{current}");
        *last = Some(current);
    }
}

fn get_single(mon: &str) -> Result<String> {
    let monitors: Vec<Monitor> = query("monitors")?;
    let active_workspace_id = monitors
        .iter()
        .find(|m| m.name == mon)
        .ok_or_else(|| {
            log::error!("No monitor found with name: {}", mon);
            "No monitor found"
        })?
        .active_workspace
        .id;
    let workspaces: Vec<Workspace> = query("workspaces")?;
    let title = workspaces
        .into_iter()
        .find(|w| w.id == active_workspace_id)
        .ok_or_else(|| {
            log::warn!("No workspace found with ID: {}", active_workspace_id);
            "No workspace found"
        })?
        .last_window_title;
    Ok(title)
}

fn get_all() -> Result<String> {
    let monitors: Vec<Monitor> = query("monitors")?;
    let workspaces: Vec<Workspace> = query("workspaces")?;
    let clients: Vec<Client> = query("clients")?;
    let mut out_monitors: Vec<MonitorCustom> = Vec::new();
    for monitor in monitors {
        let workspace = workspaces
            .iter()
            .find(|w| w.id == monitor.active_workspace.id)
            .ok_or_else(|| {
                log::error!(
                    "No active workspace found. ID: {}",
                    monitor.active_workspace.id
                );
                "No active workspace found"
            })?;

        let (title, initial_title) = match clients.iter().find(|c| c.address == workspace.last_window) {
            Some(c) => (c.title.clone(), c.initial_title.clone()),
            None => (String::new(), String::new()),
        };

        out_monitors.push(MonitorCustom {
            name: monitor.name,
            title,
            initial_title,
        });
    }
    Ok(json!(out_monitors).to_string())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    //check args
    if args.len() != 2 || args[1].eq("-h") || args[1].eq("--help") {
        println!("{HELP}");
        std::process::exit(0);
    }

    let _logger = match Logger::try_with_str("info") {
        Ok(logger) => {
            match logger
                .log_to_file(
                    FileSpec::default()
                        .directory("/tmp")
                        .basename("hyprland-activewindow"),
                )
                .start()
            {
                Ok(logger) => logger,
                Err(e) => {
                    println!("Unable to start logger: {}", e);
                    std::process::exit(1)
                }
            };
        }
        Err(e) => {
            println!("Unable to initialise logger: {}", e);
            std::process::exit(1)
        }
    };

    let mon = args[1].to_string();
    let monitors: Vec<Monitor> = query("monitors").unwrap_or_else(|err| {
        log::error!("Unable to get monitors: {}", err);
        std::process::exit(1)
    });
    if !monitors.iter().any(|m| m.name == mon) && mon != "_" {
        log::error!("Unable to find monitor {mon}");
        std::process::exit(0);
    }

    log::info!("Started with arg {}", mon);
    let mut last = None;
    output(&mon, &mut last);

    // Create a event listener
    let events = BufReader::new(UnixStream::connect(socket_path(".socket2.sock")?)?);
    for line in events.lines() {
        let line = line?;
        let event = line.split_once(">>").map_or(line.as_str(), |(name, _)| name);
        if REFRESH_EVENTS.contains(&event) {
            output(&mon, &mut last);
        }
    }
    Err("Hyprland event socket closed".into())
}
