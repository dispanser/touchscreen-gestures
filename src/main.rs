use std::{
    fs::OpenOptions,
    mem,
    os::{
        fd::{AsFd, IntoRawFd as _, OwnedFd},
        unix::fs::OpenOptionsExt as _,
    },
    path::Path,
    time::Duration,
};

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
use touchscreen_gestures::actions::ActionHandler;
use touchscreen_gestures::error::Result;
use touchscreen_gestures::touch::{classifier::classify_gesture, TouchState};
use touchscreen_gestures::{
    accel::{Orientation, OrientationSensor, ZbusOMeter},
    // xrandr_handler,
};

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
    if has_gesture_device(&mut interface) {
        let mut eh = EventHandler::new(Config::my_config()).await?;
        eh.main_loop(&mut interface).await?;
    } else {
        log::error!("no device with touch capabilities available")
    }
    Ok(())
}

fn has_gesture_device(input: &mut Libinput) -> bool {
    input.dispatch().unwrap();
    for event in input.clone() {
        if let Event::Device(e) = event {
            if e.device().has_capability(DeviceCapability::Touch) {
                log::info!(name = e.device().name(); "detected touch-capable device");
                return true;
            }
        }
        input.dispatch().unwrap();
    }
    false
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

struct EventHandler {
    touch_state: TouchState,
    action_handler: ActionHandler,
    config: Config,
    orientation_sensor: Box<dyn OrientationSensor>,
    orientation: Orientation,
}

impl EventHandler {
    pub async fn new(config: Config) -> Result<Self> {
        let mut orientation_sensor = Box::new(ZbusOMeter::try_new().await?);
        let orientation = orientation_sensor.orientation().await?;
        info!("orientation at start: {orientation:?}");
        Ok(Self {
            touch_state: TouchState::default(),
            action_handler: ActionHandler::new()?,
            config,
            orientation_sensor,
            orientation,
        })
    }

    pub async fn main_loop(&mut self, input: &mut Libinput) -> Result<()> {
        let fds = PollFd::new(input.as_fd(), PollFlags::POLLIN);
        let poll_interval = PollTimeout::try_from(Duration::from_millis(500)).unwrap();
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
            self.orientation = orientation;
        }
        Ok(())
    }
}
