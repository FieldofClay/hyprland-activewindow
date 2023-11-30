use hyprland::data::{Monitors, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::shared::HyprData;
use hyprland::Result;
use std::env;
use serde::Serialize;
use serde_json::json;

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
}

fn print_title(mon: &String) {
    let active_workspace_id = Monitors::get()
        .expect("unable to get monitors")
        .find(|m| m.name == mon.to_string())
        .unwrap()
        .active_workspace
        .id;
    let title = Workspaces::get()
        .expect("unable to get workspaces")
        .find(|w| w.id == active_workspace_id)
        .unwrap()
        .last_window_title;
    println!("{}", title);
}

fn print_all() {
    let monitors = Monitors::get().expect("unable to get monitors");
    let mut out_monitors: Vec<MonitorCustom> = Vec::new();
    for monitor in monitors {
        let title = Workspaces::get()
        .expect("unable to get workspaces")
        .find(|w| w.id == monitor.active_workspace.id)
        .unwrap()
        .last_window_title;
        let mc: MonitorCustom = MonitorCustom {
            name: monitor.name,
            title,
        };
        out_monitors.push(mc);
    }
    println!("{}", json!(out_monitors).to_string());
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    //check args
    if args.len() != 2 || args[1].eq("-h") || args[1].eq("--help") {
        println!("{HELP}");
        std::process::exit(0);
    }
    let mon = args[1].to_string();
    let mon_object = Monitors::get()
    .expect("unable to get monitors")
    .find(|m| m.name == mon);
    if mon_object.is_none() && mon != "_" {
            println!("Unable to find monitor {mon}");
            std::process::exit(0);    
    }
    if mon == "_" {
        print_all();
    } else {
        print_title(&mon);
    }
    // Create a event listener
    let mut event_listener = EventListener::new();
    let mon2 = mon.clone();
    let mon3 = mon.clone();
    event_listener.add_active_window_change_handler(move |_, state| {
        if mon.eq(&state.active_monitor) {
            print_title(&mon);
        } else if mon == "_" {
            print_all();
        }
    });
    event_listener.add_window_close_handler(move |_, state| {
        if mon2.eq(&state.active_monitor) {
            print_title(&mon2);
        } else if mon2 == "_" {
            print_all();
        }
    });
    event_listener.add_workspace_change_handler(move |_, state| {
        if mon3.eq(&state.active_monitor) {
            print_title(&mon3);
        } else if mon3 == "_" {
            print_all();
        }
    });

    event_listener.start_listener()
}
