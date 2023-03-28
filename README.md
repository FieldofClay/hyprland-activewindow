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
### Building from source
```
git clone https://github.com/FieldofClay/hyprland-activewindow.git
cd hyprland-activewindow
cargo build --release
```

## Usage
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
(deflisten window0 "hyprland-activewindow `hyprctl monitors -j | jq -r \".[0].name\"`")
(defwidget title0 []
    (label :text "${window0}"))

(deflisten window1 "hyprland-activewindow `hyprctl monitors -j | jq -r \".[1].name\"`")
(defwidget title1 []
    (label :text "${window1}"))
```