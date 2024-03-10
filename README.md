# hyprland-activewindow
A multi-monitor aware Hyprland active window title outputer. Follows the specified monitor and outputs the current active window title. Designed to be used with [Eww](https://github.com/elkowar/eww), but may function with other bars.

## Installation Instructions
### Dependencies
[Hyprland](https://github.com/hyprwm/Hyprland)
### Arch Linux
Arch users can install from AUR using your favourite package manager.
```
  pikaur -S hyprland-activewindow
```
### Crates.io
It can be installed directly from [crates.io](https://crates.io) with cargo.
```
  cargo install hyprland-activewindow
```
### Building from source
```
git clone https://github.com/FieldofClay/hyprland-activewindow.git
cd hyprland-activewindow
cargo build --release
```

## Usage
### Basic Mode
Pass the name of the monitor to follow as the only argument. It will then follow that monitor and output the active window title to stdout.
```
./hyprland-activewindow eDP-1
```
You can get the names of your monitors by running:
```
hyprctl monitors -j
```

It can be used as a title widget in Eww with config similar to below.
```yuck
(deflisten window0 "hyprland-activewindow eDP-1")
(defwidget title0 []
    (label :text "${window0}"))

(deflisten window1 "hyprland-activewindow DP-1")
(defwidget title1 []
    (label :text "${window1}"))

(defwidget bar0 []
  (box
    (window0)
  )
)

(defwidget bar1 []
  (box
    (window1)
  )
)
```

### Advanced Mode
Pass the wildcard "_" as the only argument and it will follow all monitors and output active window title information in json to stdout.
```
./hyprland-activewindow _
```
The output will be a json array of each monitors name and the active window's title & initial title.
```json
[{"initial_title":"Alacritty","name":"eDP-1","title":"~/hyprland-activewindow"},{"initial_title":"VSCodium","name":"DP-1","title":"main.rs - hyprland-activewindow (Workspace) - VSCodium"}]
```
This allows simplified Eww config similar to this:
```yuck
(deflisten windows "hyprland-activewindow _")

(defwidget window [monitor]
  (box
    (label :text "${windows['${monitor}'].title}")
  )
)

(defwidget bar0 []
  (box
    (window :monitor 0)
  )
)

(defwidget bar1 []
  (box
    (window :monitor 1)
  )
)
```
