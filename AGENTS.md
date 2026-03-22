# Touchscreen Gestures Daemon

Linux daemon detecting multi-finger touchscreen gestures and executing configured actions. Dynamically adjusts screen orientation via accelerometer input.

## Key Features

- Detects 8-directional gestures (Up, Down, Left, Right, 4 diagonals) with Hold patterns
- Supports three action types: keyboard sequences, shell scripts, and screen rotation commands
- Dynamic screen rotation detection via accelerometer (4 orientation states)
- Configurable via TOML files with compact gesture pattern syntax
- Event-driven architecture using libinput for touch input
