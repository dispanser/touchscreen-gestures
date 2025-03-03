use crate::accel::Orientation;

use super::{Coordinate, FingerState};

// "resolution" is 1000x1000, so
const NO_MOVE_THRESHOLD: i16 = 20;
const S_MOVE_THRESHOLD: i16 = 300;
const EDGE_THRESHOLD: u16 = 50; // Pixels from screen edge to consider as edge

pub type Gesture = Vec<FingerPattern>;

fn detect_edge(coord: &Coordinate) -> Edge {
    if coord.x <= EDGE_THRESHOLD {
        Edge::Left
    } else if coord.x >= 1000 - EDGE_THRESHOLD {
        Edge::Right
    } else if coord.y <= EDGE_THRESHOLD {
        Edge::Top
    } else if coord.y >= 1000 - EDGE_THRESHOLD {
        Edge::Bottom
    } else {
        Edge::None
    }
}

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
pub enum Edge {
    None,
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FingerPattern {
    Hold {
        origin: Edge,
    },
    Move {
        direction: Direction,
        size: Size,
        origin: Edge,
    },
}

impl FingerPattern {
    pub fn new_move(direction: Direction, size: Size, origin: Edge) -> Self {
        FingerPattern::Move {
            direction,
            size,
            origin,
        }
    }

    pub fn apply_transformation(self, orientation: Orientation) -> Self {
        let transform_edge = |edge: Edge| match (edge, orientation) {
            (Edge::None, _) => Edge::None,
            // Normal orientation - no change
            (e, Orientation::Normal | Orientation::Unknown) => e,
            // 90° clockwise
            (Edge::Left, Orientation::LeftUp) => Edge::Top,
            (Edge::Top, Orientation::LeftUp) => Edge::Right,
            (Edge::Right, Orientation::LeftUp) => Edge::Bottom,
            (Edge::Bottom, Orientation::LeftUp) => Edge::Left,
            // 90° counter-clockwise
            (Edge::Left, Orientation::RightUp) => Edge::Bottom,
            (Edge::Bottom, Orientation::RightUp) => Edge::Right,
            (Edge::Right, Orientation::RightUp) => Edge::Top,
            (Edge::Top, Orientation::RightUp) => Edge::Left,
            // 180° rotation
            (Edge::Left, Orientation::BottomUp) => Edge::Right,
            (Edge::Right, Orientation::BottomUp) => Edge::Left,
            (Edge::Top, Orientation::BottomUp) => Edge::Bottom,
            (Edge::Bottom, Orientation::BottomUp) => Edge::Top,
        };

        match self {
            FingerPattern::Hold { origin } => FingerPattern::Hold {
                origin: transform_edge(origin),
            },
            FingerPattern::Move {
                direction,
                size,
                origin,
            } => {
                let rotation_steps = match orientation {
                    Orientation::Normal | Orientation::Unknown => 0,
                    Orientation::LeftUp => 2,   // 90° clockwise
                    Orientation::RightUp => -2, // 90° counter-clockwise
                    Orientation::BottomUp => 4, // 180°
                };
                FingerPattern::Move {
                    direction: direction.rotate(rotation_steps),
                    size,
                    origin: transform_edge(origin),
                }
            }
        }
    }
}

#[cfg(test)]
mod transform_tests {
    use super::*;

    #[test]
    fn test_hold_transformation() {
        // Test edge transformations for Hold pattern
        let hold_left = FingerPattern::Hold { origin: Edge::Left };
        assert_eq!(
            hold_left.apply_transformation(Orientation::LeftUp),
            FingerPattern::Hold { origin: Edge::Top }
        );
        assert_eq!(
            hold_left.apply_transformation(Orientation::RightUp),
            FingerPattern::Hold {
                origin: Edge::Bottom
            }
        );
        assert_eq!(
            hold_left.apply_transformation(Orientation::BottomUp),
            FingerPattern::Hold {
                origin: Edge::Right
            }
        );

        // Test that center holds remain unchanged
        let hold_center = FingerPattern::Hold { origin: Edge::None };
        assert_eq!(
            hold_center.apply_transformation(Orientation::LeftUp),
            FingerPattern::Hold { origin: Edge::None }
        );
    }

    #[test]
    fn test_left_up_transformation() {
        let move_up = FingerPattern::new_move(Direction::Up, Size::L, Edge::None);
        assert_eq!(
            move_up.apply_transformation(Orientation::LeftUp),
            FingerPattern::new_move(Direction::Right, Size::L, Edge::None)
        );

        let move_upleft = FingerPattern::new_move(Direction::UpLeft, Size::S, Edge::Left);
        assert_eq!(
            move_upleft.apply_transformation(Orientation::LeftUp),
            FingerPattern::new_move(Direction::UpRight, Size::S, Edge::Top)
        );
    }

    #[test]
    fn test_bottom_up_transformation() {
        let move_right = FingerPattern::new_move(Direction::Right, Size::S, Edge::Top);
        assert_eq!(
            move_right.apply_transformation(Orientation::BottomUp),
            FingerPattern::new_move(Direction::Left, Size::S, Edge::Bottom)
        );

        let move_downright = FingerPattern::new_move(Direction::DownRight, Size::L, Edge::Right);
        assert_eq!(
            move_downright.apply_transformation(Orientation::BottomUp),
            FingerPattern::new_move(Direction::UpLeft, Size::L, Edge::Left)
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
    let origin_edge = detect_edge(&finger.start_position);

    if dx.abs() <= NO_MOVE_THRESHOLD && dy.abs() <= NO_MOVE_THRESHOLD {
        return FingerPattern::Hold {
            origin: origin_edge,
        };
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

    FingerPattern::new_move(direction, size, origin_edge)
}

#[cfg(test)]
mod test {
    use crate::{
        accel::Orientation,
        touch::{
            classifier::{Direction, Edge, Size},
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

    fn make_finger_state_at(x: u16, y: u16) -> FingerState {
        FingerState {
            start_time: 0,
            start_position: Coordinate { x, y },
            last_position: Coordinate { x, y },
            active: true,
        }
    }

    fn make_finger_state_with_movement(
        start_x: u16,
        start_y: u16,
        dx: i16,
        dy: i16,
    ) -> FingerState {
        FingerState {
            start_time: 0,
            start_position: Coordinate {
                x: start_x,
                y: start_y,
            },
            last_position: Coordinate {
                x: (start_x as i16 + dx) as u16,
                y: (start_y as i16 + dy) as u16,
            },
            active: true,
        }
    }

    #[test]
    fn test_edge_detection() {
        assert_eq!(
            detect_finger_pattern(&make_finger_state_at(10, 500)),
            FingerPattern::Hold { origin: Edge::Left }
        );

        assert_eq!(
            detect_finger_pattern(&make_finger_state_at(990, 500)),
            FingerPattern::Hold {
                origin: Edge::Right
            }
        );

        assert_eq!(
            detect_finger_pattern(&make_finger_state_with_movement(10, 500, 100, 0)),
            FingerPattern::new_move(Direction::Right, Size::S, Edge::Left)
        );
    }

    #[test]
    fn test_rotation_bounds() {
        // Test that multiple rotations don't exceed bounds
        let move_up = FingerPattern::new_move(Direction::Up, Size::L, Edge::None);

        // Test clockwise rotations (positive steps)
        assert_eq!(
            move_up.apply_transformation(Orientation::LeftUp), // +2
            FingerPattern::Move {
                direction: Direction::Right,
                size: Size::L,
                origin: Edge::None,
            }
        );
        assert_eq!(
            move_up.apply_transformation(Orientation::BottomUp), // +4
            FingerPattern::new_move(Direction::Down, Size::L, Edge::None)
        );

        // Test counter-clockwise rotations (negative steps)
        assert_eq!(
            move_up.apply_transformation(Orientation::RightUp), // -2
            FingerPattern::new_move(Direction::Left, Size::L, Edge::None)
        );
    }

    #[test]
    fn test_hold_patterns() {
        assert_eq!(
            detect_finger_pattern(&make_finger_state(0, 0)),
            FingerPattern::Hold { origin: Edge::Left },
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-9, -20)),
            FingerPattern::Hold { origin: Edge::Left },
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(13, 0)),
            FingerPattern::Hold { origin: Edge::Left },
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(20, -20)),
            FingerPattern::Hold { origin: Edge::Left },
        );
    }

    #[test]
    fn test_move_patterns() {
        assert_eq!(
            detect_finger_pattern(&make_finger_state(0, -521)),
            FingerPattern::new_move(Direction::Up, Size::L, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(399, 899)),
            FingerPattern::new_move(Direction::Down, Size::L, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(39, 19)),
            FingerPattern::new_move(Direction::Right, Size::S, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-39, -19)),
            FingerPattern::new_move(Direction::Left, Size::S, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(70, 36)),
            FingerPattern::new_move(Direction::DownRight, Size::S, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(11, -21)),
            FingerPattern::new_move(Direction::UpRight, Size::S, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-337, -193)),
            FingerPattern::new_move(Direction::UpLeft, Size::L, Edge::None),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-11, 21)),
            FingerPattern::new_move(Direction::DownLeft, Size::S, Edge::Left),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-980, 0)),
            FingerPattern::new_move(Direction::Left, Size::L, Edge::Right),
        );
    }
}
