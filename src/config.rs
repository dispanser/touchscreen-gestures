use std::collections::HashMap;

use touchscreen_gestures::actions::{keyboard::KeySequence, Action};

use touchscreen_gestures::touch::classifier::{Direction::*, FingerPattern::*, Gesture, Size::*};

#[derive(Debug)]
pub struct Config {
    pub actions: HashMap<Gesture, Action>,
}

impl Config {
    pub fn my_config() -> Self {
        Config {
            actions: [
                (
                    vec![Move(Down, S), Move(Down, S), Move(Down, S), Move(Down, S)],
                    script(vec!["/run/current-system/sw/bin/light", "-A", "10"]),
                ),
                (
                    vec![Move(Up, S), Move(Up, S), Move(Up, S), Move(Up, S)],
                    script(vec!["/run/current-system/sw/bin/light", "-U", "10"]),
                ),
                (
                    vec![Move(Left, S), Move(Left, S)],
                    keys(vec!["ctrl - h"]), // "backward" in qute
                ),
                (
                    vec![Move(Right, S), Move(Right, S)],
                    keys(vec!["ctrl - l"]), // "forward" in qute
                ),
                (
                    vec![Move(Right, L)],
                    keys(vec!["r"]), // "forward" in qute
                ),
                (
                    vec![Move(Left, L)],
                    keys(vec!["l"]), // "forward" in qute
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
