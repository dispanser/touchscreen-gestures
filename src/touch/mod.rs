use input::event::{
    touch::{TouchEventPosition, TouchEventSlot as _, TouchEventTrait},
    TouchEvent,
};

pub mod classifier;

#[derive(Clone, Debug, Copy)]
pub struct Coordinate {
    x: u16,
    y: u16,
}

impl Coordinate {
    fn from_event(event: &impl TouchEventPosition) -> Self {
        Self {
            x: event.x_transformed(1000) as u16,
            y: event.y_transformed(1000) as u16,
        }
    }

    fn delta_from(&self, from: &Self) -> (i16, i16) {
        (self.x as i16 - from.x as i16, self.y as i16 - from.y as i16)
    }
}

// The state of an in-progress touch event.
// The event is considered finished when all end positions are determined.
#[derive(Debug, Default, Clone)]
pub struct TouchState {
    /// number of fingers involved in the event
    pub active_fingers: u32,

    /// state for each individual slot (finger)
    pub fingers: Vec<FingerState>,
}

impl TouchState {
    pub fn update(&mut self, event: &TouchEvent) {
        match event {
            TouchEvent::Down(ref down_event) => {
                let slot = down_event.slot().unwrap();
                let slot = slot as usize;
                self.active_fingers += 1;
                let coord = Coordinate::from_event(down_event);
                if self.fingers.len() > slot {
                    log::debug!("re-touching previously seen finger '{}'", slot);
                    return;
                } else {
                    log::debug!("adding finger '{}'", slot);
                    // a new finger!
                    let state = FingerState {
                        start_time: down_event.time(),
                        start_position: coord,
                        last_position: coord,
                        active: true,
                    };
                    self.fingers.push(state);
                }
                log::trace!("down event");
            }
            TouchEvent::Up(ref up_event) => {
                let slot = up_event.slot().unwrap();
                if slot < self.fingers.len() as u32 {
                    let finger = &mut self.fingers[slot as usize];
                    finger.active = false;
                    self.active_fingers -= 1;
                    log::debug!("finger '{}' up", slot);
                } else {
                    log::error!(
                        "[U]: untracked slot '{}', fingers: {}",
                        slot,
                        self.fingers.len()
                    );
                }
            }
            TouchEvent::Motion(ref motion) => {
                let slot = motion.slot().unwrap();
                log::debug!(
                    "[{}, {}]",
                    motion.x_transformed(1000) as u16,
                    motion.y_transformed(1000) as u16
                );
                if slot < self.fingers.len() as u32 {
                    let finger = &mut self.fingers[slot as usize];
                    if !finger.active {
                        finger.active = true;
                    }
                    finger.last_position = Coordinate::from_event(motion);
                } else {
                    log::error!(
                        "[M]: untracked slot '{}', fingers: {}",
                        slot,
                        self.fingers.len()
                    );
                }
            }
            TouchEvent::Frame(ref frame) => {
                // "Signals the end of a set of touchpoints at one device sample time."
                log::trace!(time = frame.time(); "frame event");
            }
            _ => log::warn!("unhandled touch event {event:?}"),
        }
    }

    pub fn completed(&self) -> bool {
        self.active_fingers == 0 && !self.fingers.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct FingerState {
    pub start_time: u32,
    pub start_position: Coordinate,
    pub last_position: Coordinate,
    pub active: bool,
}
