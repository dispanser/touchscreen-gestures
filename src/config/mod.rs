use std::collections::HashMap;

use touchscreen_gestures::actions::Cmd;
use touchscreen_gestures::actions::{keyboard::KeySequence, Action};

use touchscreen_gestures::touch::classifier::Edge;
use touchscreen_gestures::touch::classifier::{Direction::*, FingerPattern, Gesture, Size::*};

#[derive(Debug)]
pub struct Config {
    pub poll_interval_ms: u64,
    pub actions: HashMap<Gesture, Action>,
}

impl Config {
    pub fn my_config(poll_interval_ms: u64) -> Self {
        Config {
            poll_interval_ms,
            actions: [
                (
                    gestures(vec![
                        FingerPattern::new_move(Up, S, Edge::Bottom),
                        FingerPattern::new_move(Up, S, Edge::Bottom),
                    ]),
                    Action::Cmd(Cmd::InternalScreen),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                    ]),
                    script(vec!["/run/current-system/sw/bin/light", "-A", "10"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                    ]),
                    script(vec!["/run/current-system/sw/bin/light", "-U", "10"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                        FingerPattern::new_move(Down, L, Edge::None),
                    ]),
                    script(vec!["/run/current-system/sw/bin/light", "-A", "30"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                        FingerPattern::new_move(Up, L, Edge::None),
                    ]),
                    script(vec!["/run/current-system/sw/bin/light", "-U", "30"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                        FingerPattern::new_move(Down, S, Edge::None),
                    ]),
                    keys(vec!["r"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                        FingerPattern::new_move(Up, S, Edge::None),
                    ]),
                    keys(vec!["x"]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Left, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                        FingerPattern::new_move(Left, S, Edge::None),
                    ]),
                    keys(vec!["alt - j"]), // previous tab
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                    ]),
                    keys(vec!["alt - k"]), // next tab
                ),
                (
                    gestures(vec![FingerPattern::new_move(Left, S, Edge::Right)]),
                    keys(vec!["ctrl - l"]), // "forward" in qute
                ),
                (
                    gestures(vec![FingerPattern::new_move(Right, S, Edge::Left)]),
                    keys(vec!["ctrl - h"]), // "backward" in qute
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Up, S, Edge::Left),
                        FingerPattern::new_move(Up, S, Edge::Right),
                    ]),
                    script(vec![
                        "/etc/profiles/per-user/pi/bin/dbus-send",
                        "--type=method_call",
                        "--dest=org.onboard.Onboard",
                        "/org/onboard/Onboard/Keyboard",
                        "org.onboard.Onboard.Keyboard.ToggleVisible",
                    ]),
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Down, S, Edge::Left),
                        FingerPattern::new_move(Down, S, Edge::Right),
                    ]),
                    script(vec![
                        "/nix/store/hp5ca5wkhkxvldva26yqmy52azczl1sq-onboard-1.4.1/bin/onboard",
                        "-l",
                        "/home/pi/src/github/dispanser/dot-files/configs/onboard/mine.onboard",
                    ]),
                ),
            ]
            .into_iter()
            .collect(),
        }
    }
}

fn gestures(mut patterns: Vec<FingerPattern>) -> Vec<FingerPattern> {
    patterns.sort();
    patterns
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
