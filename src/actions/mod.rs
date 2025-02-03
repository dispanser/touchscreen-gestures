use std::process::Command;

use keyboard::{KeySequence, Keyboard};

use super::error::Result;

pub mod keyboard;

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
                log::info!("running {} with {:?}", args[0], &args[1..]);
                log::info!("result: {_child:?}");
            }
        }
    }
}

#[derive(Debug)]
pub enum Action {
    KeySeq(KeySequence),
    Script(Vec<String>),
}
