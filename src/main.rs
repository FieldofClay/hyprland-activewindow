use hyprland::data::{Monitor, Monitors, Workspace, Workspaces};
use hyprland::event_listener::EventListenerMutable as EventListener;
use hyprland::shared::{HResult, HyprData};
use std::env;
use std::{thread, time};

const HELP: &str = "\
hyprland-activewindow: a multi monitor aware active hyprland window title reporter, designed to be used with eww.

USAGE:
  hyprland-activewindow MONITOR

FLAGS:
  -h, --help            Prints help information

ARGS:
  <MONITOR>             Monitor to report active window title on
";

fn main() -> HResult<()> {
    let args: Vec<String> = env::args().collect();
    //check args
    if args.len() != 2 || args[1].eq("-h") || args[1].eq("--help") {
        println!("{HELP}");
        std::process::exit(0);
    }
    // Create a event listener
    let mut event_listener = EventListener::new();
    event_listener.add_active_window_change_handler(move |data, state| {
        let mon = &args[1];
        use hyprland::event_listener::WindowEventData;
        let title = match data {
            Some(WindowEventData(_, title)) => format!("{title}"),
            None => "".to_string(),
        };
        if mon.eq(&state.active_monitor) {
            if title.ne("") {
                println!("{}", title);
            } else {
                //when changing workspaces, hyprland will output an empty title
                //here we check if that workspace is actually empty or not
                //and only updating the active title if it is
                
                //just wait a moment for config to update
                thread::sleep(time::Duration::from_millis(100));
                let ws_id = Monitors::get()
                    .expect("unable to get monitors")
                    .filter(|m| m.name.eq(mon))
                    .collect::<Vec<Monitor>>()[0]
                    .active_workspace
                    .id;

                let window_cnt = Workspaces::get()
                    .expect("unable to get workspaces")
                    .filter(|w| w.id.eq(&ws_id))
                    .collect::<Vec<Workspace>>()[0]
                    .windows;

                if window_cnt == 0u8 {
                    println!("{}", title);
                }
            }
        }
    });

    event_listener.start_listener()
}
