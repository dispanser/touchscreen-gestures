use std::collections::HashMap;

use touchscreen_gestures::actions::{keyboard::KeySequence, Action};

use crate::touch::classifier::{Direction::*, FingerPattern::*, Gesture, Size::*};

#[derive(Debug)]
pub struct Config {
    pub actions: HashMap<Gesture, Action>,
}

impl Config {
    pub fn my_config() -> Self {
        Config {
            actions: [
                (
                    vec![Move(Down, M), Move(Down, M), Move(Down, M), Move(Down, M)],
                    script(vec!["/run/current-system/sw/bin/light", "-A", "10"]),
                ),
                (
                    vec![Move(Up, M), Move(Up, M), Move(Up, M), Move(Up, M)],
                    script(vec!["/run/current-system/sw/bin/light", "-D", "10"]),
                ),
                (
                    vec![Move(Left, M), Move(Left, M)],
                    keys(vec!["S-L"]), // "forward" in qute
                ),
                (
                    vec![Move(Right, M), Move(Right, M)],
                    keys(vec!["S-H"]), // "backward" in qute
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}

fn script(cmd: Vec<&str>) -> Action {
    Action::Script(cmd.into_iter().map(Into::into).collect())
}

fn keys(key_steps: Vec<&str>) -> Action {
    let seq = key_steps
        .into_iter()
        .fold(KeySequence::default(), |seq, step| {
            seq.parse_step(step).expect("Invalid key sequence")
        });
    Action::KeySeq(seq)
}
