use std::process::Command;

use keyboard::{KeySequence, Keyboard};

use super::error::Result;

pub mod keyboard;

use std::sync::mpsc::Sender;

pub struct ActionHandler {
    keyboard: Keyboard,
    cmd_tx: Sender<Cmd>,
}

impl ActionHandler {
    pub fn new(cmd_tx: Sender<Cmd>) -> Result<Self> {
        let keyboard = Keyboard::new()?;
        Ok(Self { keyboard, cmd_tx })
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
            Action::Cmd(cmd) => {
                if let Err(e) = self.cmd_tx.send(cmd.clone()) {
                    log::error!("Failed to send command: {e}");
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Action {
    KeySeq(KeySequence),
    Script(Vec<String>),
    Cmd(Cmd),
}

#[derive(Debug, Clone)]
pub enum Cmd {
    InternalScreen,
    ExternalScreen,
    BothScreens,
    ResetScreens,
}
