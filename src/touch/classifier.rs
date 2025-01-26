use super::FingerState;

const NO_MOVE_THRESHOLD: i16 = 20;
const S_MOVE_THRESHOLD: i16 = 200;
const M_MOVE_THRESHOLD: i16 = 500;
const L_MOVE_THRESHOLD: i16 = 800;

pub type Gesture = Vec<FingerPattern>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Size {
    S,
    M,
    L,
    XL,
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

impl FingerPattern {}

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
        x if x >= L_MOVE_THRESHOLD => Size::XL,
        x if x >= M_MOVE_THRESHOLD => Size::L,
        x if x >= S_MOVE_THRESHOLD => Size::M,
        _ => Size::S,
    };

    FingerPattern::Move(direction, size)
}

#[cfg(test)]
mod test {
    use crate::touch::{
        classifier::{Direction, Size},
        Coordinate, FingerState,
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
            FingerPattern::Move(Direction::Down, Size::XL),
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
            FingerPattern::Move(Direction::UpLeft, Size::M),
        );
        assert_eq!(
            detect_finger_pattern(&make_finger_state(-11, 21)),
            FingerPattern::Move(Direction::DownLeft, Size::S),
        );
    }
}
