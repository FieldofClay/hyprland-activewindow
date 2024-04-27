use flexi_logger::{FileSpec, Logger};
use hyprland::data::{Clients, Monitors, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::shared::{HyprData, HyprError};
use hyprland::Result;
use log;
use serde::Serialize;
use serde_json::json;
use std::env;
use std::sync::Arc;

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

fn print(mon: &str) {
    if mon == "_" {
        print_all().unwrap_or_else(|err| log::error!("Error printing all: {}", err));
    } else {
        print_single(mon).unwrap_or_else(|err| log::error!("Error printing single: {}", err));
    }
}

fn print_single(mon: &str) -> Result<()> {
    let active_workspace_id = Monitors::get()?
        .find(|m| m.name == mon.to_string())
        .ok_or_else(|| {
            log::error!("No monitor found with name: {}", mon);
            HyprError::NotOkDispatch("No monitor found".to_string())
        })?
        .active_workspace
        .id;
    let title = Workspaces::get()?
        .find(|w| w.id == active_workspace_id)
        .ok_or_else(|| {
            log::warn!("No workspace found with ID: {}", active_workspace_id);
            HyprError::NotOkDispatch("No workspace found".to_string())
        })?
        .last_window_title;
    println!("{}", title);
    Ok(())
}

fn print_all() -> Result<()> {
    let monitors = Monitors::get()?;
    let mut out_monitors: Vec<MonitorCustom> = Vec::new();
    for monitor in monitors {
        let workspace = Workspaces::get()?
            .find(|w| w.id == monitor.active_workspace.id)
            .ok_or_else(|| {
                log::error!(
                    "No active workspace found. ID: {}",
                    monitor.active_workspace.id
                );
                HyprError::NotOkDispatch("No active workspace found".to_string())
            })?;

        let client = Clients::get()?.find(|c| c.address == workspace.last_window);

        let (title, initial_title) = match client {
            Some(c) => (c.title, c.initial_title),
            None => (String::new(), String::new()),
        };

        let mc: MonitorCustom = MonitorCustom {
            name: monitor.name,
            title: title,
            initial_title: initial_title,
        };
        out_monitors.push(mc);
    }
    println!("{}", json!(out_monitors).to_string());
    Ok(())
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

    let mon = Arc::new(String::from(args[1].to_string()));
    let mon_object = Monitors::get()
        .unwrap_or_else(|err| {
            log::error!("Unable to get monitors: {}", err);
            std::process::exit(1)
        })
        .find(|m| m.name == mon.to_string());
    if mon_object.is_none() && mon.to_string() != "_" {
        log::error!("Unable to find monitor {mon}");
        std::process::exit(0);
    }

    log::info!("Started with arg {}", mon);
    print(&mon);

    // Create a event listener
    let mut event_listener = EventListener::new();
    let mon_clone = Arc::clone(&mon);
    event_listener.add_active_window_change_handler(move |_, _| {
        print(&mon_clone);
    });
    let mon_clone = Arc::clone(&mon);
    event_listener.add_window_close_handler(move |_, _| {
        print(&mon_clone);
    });
    let mon_clone = Arc::clone(&mon);
    event_listener.add_workspace_change_handler(move |_, _| {
        print(&mon_clone);
    });
    let mon_clone = Arc::clone(&mon);
    event_listener.add_window_moved_handler(move |_, _| {
        print(&mon_clone);
    });
    let mon_clone = Arc::clone(&mon);
    event_listener.add_window_title_change_handler(move |_, _| {
        print(&mon_clone);
    });

    event_listener.start_listener()
}
