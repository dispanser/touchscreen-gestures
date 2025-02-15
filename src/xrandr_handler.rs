use crate::{accel::Orientation, error::Result};
use log::debug;
use xrandr::{Output, Rotation, XHandle, XrandrError};

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
    pub xhandle: XHandle,
    pub base_orientation: Orientation,
    pub state: State,
}

pub struct DisplayMatch {
    pub width: i32,
    pub height: i32,
}

impl DisplayHandler {
    ///  base_orientation to indicate the default orientation report of the internal screen
    pub fn try_new(base_orientation: Orientation) -> Result<Self> {
        Ok(Self {
            xhandle: XHandle::open()?,
            base_orientation,
            state: State::Internal,
        })
    }

    /// Auto-configure screen. Hardcoded for my setup, not applicable to anyone
    pub fn auto(&mut self, orientation: &Orientation) -> Result<()> {
        let outputs = self.xhandle.all_outputs()?.into_iter().collect::<Vec<_>>();

        Ok(match &self.state {
            State::Internal => outputs.iter().try_for_each(|output| {
                if output.connected {
                    if is_internal(&output) {
                        log::debug!("rotating active output {}", output.name);
                        self.xhandle
                            .set_rotation(&output, &from_orientation(orientation))
                    } else {
                        log::debug!("disabling output {}", output.name);
                        self.xhandle.disable(&output)
                    }
                } else if is_internal(&output) {
                    log::debug!("activing and rotation output {}", output.name);
                    self.xhandle.enable(&output, &from_orientation(orientation))
                } else {
                    Ok(())
                }
            }),
            _ => unimplemented!("states"),
        }?)
    }

    pub fn internal(&mut self, orientation: &Orientation) -> Result<()> {
        let active_outputs = self
            .xhandle
            .all_outputs()?
            .into_iter()
            .filter(|output| output.connected)
            .collect::<Vec<_>>();
        Ok(active_outputs.iter().try_for_each(|output| {
            let name = output.name.to_lowercase();
            if name.contains("edp") {
                debug!("enabling {name}");
                self.xhandle
                    .enable(output, &from_orientation(orientation))?;
            } else {
                debug!("disabling {name}");
                self.xhandle.disable(output)?;
            }

            Ok::<(), XrandrError>(())
        })?)
    }
}

fn is_internal(output: &Output) -> bool {
    output.name.to_lowercase().contains("edp")
}

fn from_orientation(orientation: &Orientation) -> Rotation {
    match orientation {
        Orientation::Normal => Rotation::Normal,
        Orientation::LeftUp => Rotation::Left,
        Orientation::BottomUp => Rotation::Inverted,
        Orientation::RightUp => Rotation::Right,
    }
}

pub fn test() -> Result<()> {
    let mut h = DisplayHandler::try_new(Orientation::Normal)?;
    h.internal(&Orientation::LeftUp)
}
