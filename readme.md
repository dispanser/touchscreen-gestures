# Touchscreen-gestures

Allows you to define various gestures on a touchscreen to trigger actions.

This is similar to and inspired by [lisgd](https://github.com/jjsullivan5196/lisgd), but supports a wider range of
gestures. It's also exposed as a crate (library) in addition to an executable
so it can be embedded into windoer managers or similar programs.

## Actions

- [x] create sequences of keypresses
- [x] run commands
- [x] a set of hard-coded commands that trigger some hard-coded internal actions
    - this is a workaround for the lack of an IPC interface to control the internal state20

## CLI Arguments

```bash
touchscreen-gestures --config <PATH>
```

- `--config`: path to the TOML configuration file (required)

## Configuration Format

The configuration file is a TOML file with the following structure:

```toml
# Poll interval in milliseconds
poll_interval_ms = 30

# Define actions for gestures
[[actions]]
gesture = ["U,S,B", "U,S,B"]
run = "/run/current-system/sw/bin/light -A 10"

[[actions]]
gesture = ["D,S", "D,S", "D,S"]
keys = ["r"]

[[actions]]
gesture = ["U,L", "U,L", "U,L", "U,L"]
cmd = "InternalScreen"
```

### Fields

- `poll_interval_ms`: integer, how often to poll for input events (ms)
- `gesture`: array of finger pattern strings, one per finger
- `run`: command to execute (whitespace-separated arguments)
- `keys`: array of key sequences to send (e.g., `"ctrl - h"`, `"alt - tab"`)
- `cmd`: internal command (`InternalScreen`, `ExternalScreen`, `BothScreens`, `ResetScreens`)

### Finger Pattern Format

Each finger pattern is a comma-separated string: `DIRECTION,SIZE,EDGE`

- **Direction**: `U` (up), `D` (down), `L` (left), `R` (right), `UL`, `UR`, `DL`, `DR`, `H` (hold)
- **Size**: `S` (short), `L` (long)
- **Edge** (optional): `N` (none), `T` (top), `B` (bottom), `L` (left), `R` (right), `TL`, `TR`, `BL`, `BR`

Examples:
- `"U,S,B"` - short upward swipe from bottom edge
- `"D,L"` - long downward swipe (no edge requirement)
- `"H,S,T"` - hold at top edge

## Detected Gestures
- [x] regular multi-finger gestures
    - whatever number of fingers your touch screen / driver supports
- [x] track each finger separately (e.g, one finger goes down, another up)
- [x] finger "hold": not moving is a choice!
- [/] edge gestures: detect if a finger movements starts or ends at an edge
    - currently, start edge is supported, end edge is not supported

### Potential future gestures

- gesture paths: split continuous finger movement into a path of multiple partial movements, like "`right` then `down` then `left` then `up`" (a rectangle).
- virtual knob: hold one (or more) fingers, rotate one (or more) fingers in a circle around it as if rotating a knob
    - e.g. for brightness / volume controls
    - this is somewhat at odds with the multi-path gesture above, as distinguishing between (`Right` -> `Down`) and rotation is hard
- dynamic dials: changing scales like volume / brightness dynamically while sliding

## NixOS / Home Manager Integration

### Using the Home Manager Module

Add the flake to your Home Manager configuration and enable the module:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    home-manager.url = "github:nix-community/home-manager";
    touchscreen-gestures.url = "github:dispanser/touchscreen-gestures";
    home-manager.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, home-manager, touchscreen-gestures }:
    home-manager.lib.homeManagerConfiguration {
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      modules = [
        touchscreen-gestures.homeManagerModules.default
        {
          programs.touchscreen-gestures = {
            enable = true;
            package = touchscreen-gestures.packages.x86_64-linux.default;
            config = ./touchscreen-gestures.toml;
          };
        }
      ];
    };
}
```

### Using in NixOS Configuration

For a specific user in your NixOS configuration:

```nix
{ config, pkgs, ... }:
let
  touchscreen-gestures = import (fetchGit {
    url = "https://github.com/dispanser/touchscreen-gestures.git";
    rev = "main";
  }) { };
in
{
  imports = [ touchscreen-gestures.homeManagerModules.default ];

  home-manager.users.yourusername = {
    programs.touchscreen-gestures = {
      enable = true;
      package = touchscreen-gestures.packages.${pkgs.system}.default;
      config = ./path/to/config.toml;
    };
  };
}
```

This will:
- Create a user systemd service `touchscreen-gestures.service`
- Start it automatically on login
- Restart on failure with a 5-second delay
- Set `RUST_LOG=info` for logging

