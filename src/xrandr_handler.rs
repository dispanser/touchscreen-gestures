use std::collections::HashMap;

use crate::{accel::Orientation, error::Result};
use log::debug;
use xrandr::{Rotation, XHandle, XrandrError};

pub struct DisplayHander {
    pub xhandle: XHandle,
    pub base_orientation: Orientation,
}

pub struct DisplayMatch {
    pub width: i32,
    pub height: i32,
}

impl DisplayHander {
    ///  base_orientation to indicate the default orientation report of the internal screen
    pub fn try_new(base_orientation: Orientation) -> Result<Self> {
        Ok(Self {
            xhandle: XHandle::open()?,
            base_orientation,
        })
    }

    /// Auto-configure screen. Hardcoded for my setup, not applicable to anyone
    pub fn auto(&mut self) -> Result<()> {
        let _active_monitors = self.xhandle.monitors()?;
        let _outputs_by_name: HashMap<_, _> = self
            .xhandle
            .all_outputs()?
            .into_iter()
            .filter(|output| output.connected)
            .map(|output| (output.name.clone(), output))
            .collect();

        Ok(())
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

    // fn active_outputs(&mut self) -> Result<impl Iterator<Item = &Output>> {
    //     Ok(self.xhandle.all_outputs()?.iter().filter(|output| {
    //         debug!("{}: {}", output.name, output.connected);
    //         output.connected
    //     }))
    // }
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
    let mut h = DisplayHander::try_new(Orientation::Normal)?;
    h.internal(&Orientation::LeftUp)
}
