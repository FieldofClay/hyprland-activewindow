use hyprland::data::{Monitors, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::shared::HyprData;
use hyprland::Result;
use std::env;

const HELP: &str = "\
hyprland-activewindow: a multi monitor aware active hyprland window title reporter, designed to be used with eww.

USAGE:
  hyprland-activewindow MONITOR

FLAGS:
  -h, --help            Prints help information

ARGS:
  <MONITOR>             Monitor to report active window title on
";

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

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    //check args
    if args.len() != 2 || args[1].eq("-h") || args[1].eq("--help") {
        println!("{HELP}");
        std::process::exit(0);
    }
    let mon = args[1].to_string();
    print_title(&mon);
    // Create a event listener
    let mut event_listener = EventListener::new();
    event_listener.add_active_window_change_handler(move |_, state| {
        if mon.eq(&state.active_monitor) {
            print_title(&mon);
        }
    });

    event_listener.start_listener()
}
