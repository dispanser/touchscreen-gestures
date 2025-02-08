use std::{
    fs::OpenOptions,
    mem,
    os::{
        fd::{AsFd, IntoRawFd as _, OwnedFd},
        unix::fs::OpenOptionsExt as _,
    },
    path::Path,
    process::Command,
    time::Duration,
};

use std::sync::mpsc::{self, Receiver};

use config::Config;
use input::{
    event::{EventTrait as _, TouchEvent},
    DeviceCapability, Event, Libinput, LibinputInterface,
};
use log::{debug, info};
use nix::{
    fcntl::OFlag,
    poll::{poll, PollFd, PollFlags, PollTimeout},
};
use touchscreen_gestures::touch::{classifier::classify_gesture, TouchState};
use touchscreen_gestures::{
    accel::{Orientation, OrientationSensor, ZbusOMeter},
    xrandr_handler,
};
use touchscreen_gestures::{actions::ActionHandler, error::GesturesError};
use touchscreen_gestures::{actions::Cmd, error::Result};

mod config;
// mod x11;

#[tokio::main]
async fn main() -> Result<()> {
    pretty_env_logger::init();
    // xrandr_handler::test()?;
    let mut interface = input::Libinput::new_with_udev(Interface);
    interface
        .udev_assign_seat("seat0")
        .expect("unable to assign seat to libinput interface(?)");
    if let Ok(device) = find_gesture_device(&mut interface) {
        log::info!("detected touch-capable device: {device:?}");
        let mut eh = EventHandler::new(device, Config::my_config(500)).await?;
        eh.main_loop(&mut interface).await?;
    } else {
        log::error!("no device with touch capabilities available")
    }
    Ok(())
}

fn query_device_by_name(name: String) -> Result<TouchDevice> {
    let output = Command::new("xinput")
        .args(["list", "--id-only", &name])
        .output()?;

    if !output.status.success() {
        return Err(GesturesError::DeviceNotFound(name));
    }

    let id = String::from_utf8(output.stdout)?
        .trim()
        .parse::<u32>()
        .map_err(|_| GesturesError::InvalidDeviceId(name.clone()))?;

    Ok(TouchDevice { name, id })
}

fn find_gesture_device(input: &mut Libinput) -> Result<TouchDevice> {
    input.dispatch().unwrap();
    for event in input.clone() {
        if let Event::Device(e) = event {
            if e.device().has_capability(DeviceCapability::Touch) {
                let name = e.device().name().to_owned();
                return query_device_by_name(name);
            }
        }
        input.dispatch().unwrap();
    }
    Err(GesturesError::DeviceNotFound("ANY_TOUCH".into()))
}

pub struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> std::result::Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((false) | (flags & OFlag::O_RDWR.bits() != 0))
            .write((flags & OFlag::O_WRONLY.bits() != 0) | (flags & OFlag::O_RDWR.bits() != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }

    fn close_restricted(&mut self, fd: OwnedFd) {
        nix::unistd::close(fd.into_raw_fd()).unwrap();
    }
}

#[derive(Debug)]
struct TouchDevice {
    name: String,
    id: u32,
}

struct EventHandler {
    device: TouchDevice,
    touch_state: TouchState,
    action_handler: ActionHandler,
    config: Config,
    orientation_sensor: Box<dyn OrientationSensor>,
    orientation: Orientation,
    cmd_rx: Receiver<Cmd>,
}

impl EventHandler {
    pub async fn new(gesture_device_name: TouchDevice, config: Config) -> Result<Self> {
        let mut orientation_sensor = Box::new(ZbusOMeter::try_new().await?);
        let orientation = orientation_sensor.orientation().await?;
        info!("orientation at start: {orientation:?}");

        let (cmd_tx, cmd_rx) = mpsc::channel();

        Ok(Self {
            device: gesture_device_name,
            touch_state: TouchState::default(),
            action_handler: ActionHandler::new(cmd_tx)?,
            config,
            orientation_sensor,
            orientation,
            cmd_rx,
        })
    }

    pub async fn main_loop(&mut self, input: &mut Libinput) -> Result<()> {
        let fds = PollFd::new(input.as_fd(), PollFlags::POLLIN);
        let poll_interval =
            PollTimeout::try_from(Duration::from_millis(self.config.poll_interval_ms)).unwrap();
        while let Ok(_fd) = poll(&mut [fds], poll_interval) {
            self.handle_orientation().await?;
            input.clone().dispatch().unwrap();
            for event in input.clone() {
                match event {
                    Event::Touch(e) => self.handle_touch_event(&e),
                    Event::Tablet(t) => log::info!(t:?; "tyx/tablet_event"),
                    _ => {}
                }
            }
            match self.cmd_rx.recv() {
                Ok(cmd) => debug!("received internal command: {cmd:?}"),
                Err(e) => info!("error receiving internal command: {e:?}"),
            }
        }
        Ok(())
    }

    fn handle_touch_event(&mut self, touch_event: &TouchEvent) {
        self.touch_state.update(touch_event);
        if self.touch_state.completed() {
            let finished_touch_event = mem::take(&mut self.touch_state);
            self.handle_finished_event(finished_touch_event);
        }
    }

    fn handle_finished_event(&mut self, state: TouchState) {
        let gesture: Vec<_> = classify_gesture(state.fingers)
            .into_iter()
            .map(|fp| fp.apply_transformation(self.orientation))
            .collect();
        println!("tyx/got: {gesture:?}");
        match self.config.actions.get(&gesture) {
            None => log::info!("unhandled gesture: {:?}", gesture.as_slice()),
            Some(action) => self.action_handler.run(action),
        }
    }

    async fn handle_orientation(&mut self) -> Result<()> {
        let orientation = self.orientation_sensor.orientation().await?;
        if orientation != self.orientation {
            debug!(
                "detected orientation change: {:?} -> {orientation:?}",
                self.orientation
            );

            Command::new("xinput")
                .args(["map-to-output", &self.device.id.to_string(), "eDP-1"])
                .spawn()?;
            self.orientation = orientation;
        }
        Ok(())
    }
}
