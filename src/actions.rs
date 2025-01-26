use std::{io, process::Command, time::Duration};

use crate::error::{keyboard_init_failed, GesturesError};

use super::error::Result;
use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, KeyCode, KeyEvent,
};

pub struct ActionHandler {
    keyboard: Keyboard,
}

impl ActionHandler {
    pub fn new() -> Result<Self> {
        let keyboard = Keyboard::new()?;
        Ok(Self { keyboard })
    }

    pub fn run(&mut self, action: &Action) {
        match action {
            Action::KeySeq(keys) => self.keyboard.press_sequence(keys),
            Action::Script(args) => {
                let cmd = &args[0];
                let _child = Command::new(cmd).args(args[1..].iter()).spawn();
                log::info!("running {} with {:?}", args[0], &args[1..])
            }
        }
    }
}

pub struct Keyboard {
    device: VirtualDevice,
}

impl Keyboard {
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        keys.insert(KeyCode::BTN_DPAD_UP);

        let mut device = VirtualDeviceBuilder::new()
            .map(|builder| builder.name("Fake Keyboard"))
            .and_then(|builder| builder.with_keys(&keys))
            .and_then(VirtualDeviceBuilder::build)
            .map_err(keyboard_init_failed)?;

        for path in device
            .enumerate_dev_nodes_blocking()
            .map_err(keyboard_init_failed)?
        {
            if let Ok(path) = path {
                println!("Available as {}", path.display());
            }
        }

        Ok(Self { device })
    }

    pub fn press_sequence(&mut self, keys: &Vec<String>) {
        log::info!("key sequence: {:?}", keys.as_slice())
    }

    pub fn key_press(&mut self, key: &KeyCode) {
        let down_event = *KeyEvent::new(*key, 1);
        self.device.emit(&[down_event]).unwrap();
        std::thread::sleep(Duration::from_secs(2));

        let up_event = *KeyEvent::new(*key, 0);
        self.device.emit(&[up_event]).unwrap();
        std::thread::sleep(Duration::from_secs(2));
    }
}

#[derive(Debug)]
pub enum Action {
    KeySeq(Vec<String>),
    Script(Vec<String>),
}
