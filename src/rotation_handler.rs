use crate::{accel::Orientation, error::Result};
use log::debug;
use std::process::Command;

#[allow(dead_code)]
pub enum State {
    /// Internal display only
    Internal,
    /// External display only
    External,
    /// Intrnal display below external display
    Above,
}

pub struct DisplayHandler {
    pub output_name: String,
    pub base_orientation: Orientation,
    pub state: State,
}

pub struct DisplayMatch {
    pub width: i32,
    pub height: i32,
}

impl DisplayHandler {
    /// base_orientation to indicate the default orientation report of the internal screen
    pub fn try_new(base_orientation: Orientation) -> Result<Self> {
        Ok(Self {
            output_name: String::from("eDP-1"),
            base_orientation,
            state: State::Internal,
        })
    }

    /// Auto-configure screen rotation using niri msg command
    pub fn auto(&mut self, orientation: &Orientation) -> Result<()> {
        let transform = orientation_to_niri_transform(orientation);

        debug!("rotating output {} to {}", self.output_name, transform);

        Command::new("niri")
            .args(["msg", "output", &self.output_name, "transform", transform])
            .spawn()?;

        Ok(())
    }

    pub fn internal(&mut self, orientation: &Orientation) -> Result<()> {
        self.auto(orientation)
    }
}

fn orientation_to_niri_transform(orientation: &Orientation) -> &'static str {
    match orientation {
        Orientation::Normal | Orientation::Unknown => "normal",
        Orientation::LeftUp => "90",
        Orientation::BottomUp => "180",
        Orientation::RightUp => "270",
    }
}

pub fn test() -> Result<()> {
    let mut h = DisplayHandler::try_new(Orientation::Normal)?;
    h.internal(&Orientation::LeftUp)
}
