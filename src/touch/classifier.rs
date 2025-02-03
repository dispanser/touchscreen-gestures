use crate::accel::Orientation;

use super::FingerState;

// "resolution" is 1000x1000, so
const NO_MOVE_THRESHOLD: i16 = 20;
const S_MOVE_THRESHOLD: i16 = 300;

pub type Gesture = Vec<FingerPattern>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Direction {
    Up = 0,
    UpRight = 1,
    Right = 2,
    DownRight = 3,
    Down = 4,
    DownLeft = 5,
    Left = 6,
    UpLeft = 7,
}

impl Direction {
    fn rotate(&self, steps: i8) -> Self {
        let variants = 8u8;
        let current = *self as u8;
        let rotated = (current as i8 + steps).rem_euclid(variants as i8) as u8;
        // Safety: we know the value is valid because of the modulo operation
        unsafe { std::mem::transmute(rotated) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size {
    S,
    L,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FingerPattern {
    Hold,
    // later: add (u16) because we may detect multiple taps
    Move(Direction, Size),
    // later: Tap can't be detected right now b/c all fingers are lifted at end of gesture
    // Tap,
    // later: track sequences
    // Sequence(Vec<FingerPattern>),
}

impl FingerPattern {
    pub fn apply_transformation(self, orientation: Orientation) -> Self {
        match self {
            FingerPattern::Hold => self,
            FingerPattern::Move(direction, size) => {
                let rotation_steps = match orientation {
                    Orientation::Normal => 0,
                    Orientation::LeftUp => 2,   // 90° clockwise
                    Orientation::RightUp => -2, // 90° counter-clockwise
                    Orientation::BottomUp => 4, // 180°
                };
                FingerPattern::Move(direction.rotate(rotation_steps), size)
            }
        }
    }
}

#[cfg(test)]
mod transform_tests {
    use super::*;

    #[test]
    fn test_hold_transformation() {
        let hold = FingerPattern::Hold;
        assert_eq!(hold.apply_transformation(Orientation::LeftUp), hold);
        assert_eq!(hold.apply_transformation(Orientation::RightUp), hold);
        assert_eq!(hold.apply_transformation(Orientation::BottomUp), hold);
    }

    #[test]
    fn test_left_up_transformation() {
        let move_up = FingerPattern::Move(Direction::Up, Size::L);
        assert_eq!(
            move_up.apply_transformation(Orientation::LeftUp),
            FingerPattern::Move(Direction::Right, Size::L)
        );

        let move_upleft = FingerPattern::Move(Direction::UpLeft, Size::S);
        assert_eq!(
            move_upleft.apply_transformation(Orientation::LeftUp),
            FingerPattern::Move(Direction::UpRight, Size::S)
        );
    }

    #[test]
    fn test_bottom_up_transformation() {
        let move_right = FingerPattern::Move(Direction::Right, Size::S);
        assert_eq!(
            move_right.apply_transformation(Orientation::BottomUp),
            FingerPattern::Move(Direction::Left, Size::S)
        );

        let move_downright = FingerPattern::Move(Direction::DownRight, Size::L);
        assert_eq!(
            move_downright.apply_transformation(Orientation::BottomUp),
            FingerPattern::Move(Direction::UpLeft, Size::L)
        );
    }
}

pub fn classify_gesture(fingers: impl IntoIterator<Item = FingerState>) -> Gesture {
    let mut gesture: Gesture = fingers
        .into_iter()
        .map(|finger| {
            let fp = detect_finger_pattern(&finger);
            log::info!(
                "finger {:?} @ {} -> {:?}: {:?}",
                finger.start_position,
                finger.start_time,
                finger.last_position,
                fp,
            );
            fp
        })
        .collect();
    gesture.sort();
    gesture
}

fn detect_finger_pattern(finger: &FingerState) -> FingerPattern {
    let (dx, dy) = finger.last_position.delta_from(&finger.start_position);

    if dx.abs() <= NO_MOVE_THRESHOLD && dy.abs() <= NO_MOVE_THRESHOLD {
        return FingerPattern::Hold;
    }

    let direction = if dx.abs() > dy.abs() * 2 {
        if dx > 0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if dy.abs() > dx.abs() * 2 {
        if dy > 0 {
            Direction::Down
        } else {
            Direction::Up
        }
    } else if dx > 0 {
        if dy > 0 {
            Direction::DownRight
        } else {
            Direction::UpRight
        }
    } else if dy > 0 {
        Direction::DownLeft
    } else {
        Direction::UpLeft
    };

    let size = match dx.abs().max(dy.abs()) {
        x if x >= S_MOVE_THRESHOLD => Size::L,
        _ => Size::S,
    };

    FingerPattern::Move(direction, size)
}

#[cfg(test)]
mod test {
    use crate::{
        accel::Orientation,
        touch::{
            classifier::{Direction, Size},
            Coordinate, FingerState,
        },
    };

    use super::{detect_finger_pattern, FingerPattern};

    fn make_finger_state(dx: i16, dy: i16) -> FingerState {
        let sx = 0.max(-dx) as u16;
        let sy = 0.max(-dy) as u16;
        FingerState {
            start_time: 0,
            start_position: Coordinate { x: sx, y: sy },
            last_position: Coordinate {
                x: (sx as i16 + dx) as u16,
                y: (sy as i16 + dy) as u16,
            },
            active: true,
        }
    }

    #[test]
    fn test_rotation_bounds() {
        // Test that multiple rotations don't exceed bounds
        let move_up = FingerPattern::Move(Direction::Up, Size::L);

        // Test clockwise rotations (positive steps)
        assert_eq!(
            move_up.apply_transformation(Orientation::LeftUp), // +2
            FingerPattern::Move(Direction::Right, Size::L)
        );
        assert_eq!(
            move_up.apply_transformation(Orientation::BottomUp), // +4
            FingerPattern::Move(Direction::Down, Size::L)
        );

        // Test counter-clockwise rotations (negative steps)
        assert_eq!(
            move_up.apply_transformation(Orientation::RightUp), // -2
            FingerPattern::Move(Direction::Left, Size::L)
        );

        // Test that Hold pattern remains unchanged
        let hold = FingerPattern::Hold;
        assert_eq!(hold.apply_transformation(Orientation::LeftUp), hold);
        assert_eq!(hold.apply_transformation(Orientation::RightUp), hold);
        assert_eq!(hold.apply_transformation(Orientation::BottomUp), hold);
    }

    #[test]
    fn test_hold_patterns() {
        assert_eq!(
            detect_finger_pattern(&make_finger_state(0, 0)),
            FingerPattern::Hold,
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-9, -20)),
            FingerPattern::Hold,
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(13, 0)),
            FingerPattern::Hold,
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(20, -20)),
            FingerPattern::Hold,
        );
    }

    #[test]
    fn test_move_patterns() {
        assert_eq!(
            detect_finger_pattern(&make_finger_state(0, -521)),
            FingerPattern::Move(Direction::Up, Size::L),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(399, 899)),
            FingerPattern::Move(Direction::Down, Size::L),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(39, 19)),
            FingerPattern::Move(Direction::Right, Size::S),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-39, -19)),
            FingerPattern::Move(Direction::Left, Size::S),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(70, 36)),
            FingerPattern::Move(Direction::DownRight, Size::S),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(11, -21)),
            FingerPattern::Move(Direction::UpRight, Size::S),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-337, -193)),
            FingerPattern::Move(Direction::UpLeft, Size::L),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-11, 21)),
            FingerPattern::Move(Direction::DownLeft, Size::S),
        );
    }
}
