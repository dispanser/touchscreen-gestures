use std::collections::HashMap;

use touchscreen_gestures::actions::{keyboard::KeySequence, Action};

use touchscreen_gestures::touch::classifier::Edge;
use touchscreen_gestures::touch::classifier::{Direction::*, FingerPattern, Gesture, Size::*};

#[derive(Debug)]
pub struct Config {
    pub actions: HashMap<Gesture, Action>,
}

impl Config {
    pub fn my_config() -> Self {
        Config {
            actions: [
                (
                    vec![
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                    ],
                    script(vec!["/run/current-system/sw/bin/light", "-A", "10"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                    ],
                    script(vec!["/run/current-system/sw/bin/light", "-U", "10"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                    ],
                    script(vec!["/run/current-system/sw/bin/light", "-A", "30"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                    ],
                    script(vec!["/run/current-system/sw/bin/light", "-U", "30"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                    ],
                    keys(vec!["r"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                    ],
                    keys(vec!["x"]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Left, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                    ],
                    keys(vec!["alt - j"]), // previous tab
                ),
                (
                    vec![
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                    ],
                    keys(vec!["alt - k"]), // next tab
                ),
                (
                    vec![
                        FingerPattern::new_move(Left, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                    ],
                    keys(vec!["ctrl - l"]), // "forward" in qute
                ),
                (
                    vec![
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                    ],
                    keys(vec!["ctrl - h"]), // "backward" in qute
                ),
                (
                    vec![
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                    ],
                    script(vec![
                        "/nix/store/hp5ca5wkhkxvldva26yqmy52azczl1sq-onboard-1.4.1/bin/onboard",
                        "-l",
                        "/home/pi/src/github/dispanser/dot-files/configs/onboard/mine.onboard",
                    ]),
                ),
                (
                    vec![
                        FingerPattern::new_move(Right, L, Edge::None),
                        FingerPattern::new_move(Left, L, Edge::None),
                    ],
                    script(vec!["killall", "-r", "onboard"]),
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
