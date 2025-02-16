# Touchscreen-gestures

Allows you to define various gestures on a touchscreen to trigger actions.

This is similar to and inspired by
[lisgd](https://github.com/jjsullivan5196/lisgd), but supports a wider range of
gestures. It's also exposed as a crate (library) in addition to an executable
so it can be embedded into windoer managers or similar programs.

## Actions

- [x] create sequences of keypresses
- [x] run commands
- [x] a set of hard-coded commands that trigger some hard-coded internal actions
    - this is a workaround for the lack of an IPC interface to control the internal state

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
- dynamic dials: changing scalas like volume / brightness dynamically while sliding

