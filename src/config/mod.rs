mod raw_config;

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use touchscreen_gestures::actions::Cmd;
use touchscreen_gestures::actions::{keyboard::KeySequence, Action};

use touchscreen_gestures::touch::classifier::{Direction, Edge, Size};
use touchscreen_gestures::touch::classifier::{Direction::*, FingerPattern, Gesture, Size::*};

use crate::config::raw_config::RawConfig;


#[derive(Debug)]
pub struct Config {
    pub poll_interval_ms: u64,
    pub actions: HashMap<Gesture, Action>,
}

impl Config {
   /// Load a configuration from a TOML file.
   ///
   /// The file must contain:
   /// ```toml
   /// poll_interval_ms = 30
   ///
   /// [[actions]]
   /// gesture = ["U,S,B", "U,S,B"]
   /// run = "/run/current-system/sw/bin/light -A 10"
   /// ```
   ///
   /// * `gesture` – an array of compact finger‑pattern strings (see parser below).  
   /// * `run` – a single command line; it will be split on whitespace and stored as
   ///   `Action::Script(Vec<String>)`.  
   ///   (If you later need `cmd` or `keys`, add the corresponding fields; the loader
   ///   will handle them as well.)
    pub fn load_from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let toml_str = fs::read_to_string(path)?;
        let raw: RawConfig = toml::from_str(&toml_str)?;

        let mut actions = HashMap::new();

        for entry in raw.actions {
            // ----------- parse gesture ---------------------------------
            let mut patterns = Vec::with_capacity(entry.gesture.len());
            for token in entry.gesture {
                let fp = parse_finger(&token)
                    .map_err(|e| format!("Failed to parse finger pattern `{}`: {:?}", token, e))?;
                patterns.push(fp);
            }
            // sort exactly like the original `gestures()` helper
            patterns.sort();
            let gesture = patterns;

            // ----------- build Action ----------------------------------
            let action = if let Some(run) = entry.run {
                // simple split on whitespace (no quoting handling)
                let args = split_args(&run);
                Action::Script(args)
            } else if let Some(cmd_str) = entry.cmd {
                // map string to the internal Cmd enum
                let cmd = match cmd_str.as_str() {
                    "InternalScreen" => Cmd::InternalScreen,
                    "ExternalScreen" => Cmd::ExternalScreen,
                    "BothScreens" => Cmd::BothScreens,
                    "ResetScreens" => Cmd::ResetScreens,
                    other => {
                        return Err(format!("Unknown cmd value `{}`", other).into());
                    }
                };
                Action::Cmd(cmd)
            } else if let Some(key_steps) = entry.keys {
                // reuse the existing `keys` helper (creates a KeySeq)
                Action::KeySeq(parse_key_sequence(key_steps))
            } else {
                return Err("Action entry must contain `run`, `cmd` or `keys`".into());
            };

            actions.insert(gesture, action);
        }

        Ok(Config {
            poll_interval_ms: raw.poll_interval_ms,
            actions,
        })
    }

    /// hardcoded config. Deprecated and to be removed.
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
                    keys(vec!["alt - k"]), // previous tab
                ),
                (
                    gestures(vec![
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                        FingerPattern::new_move(Right, S, Edge::None),
                    ]),
                    keys(vec!["alt - j"]), // next tab
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
                        "/run/current-system/sw/bin/dbus-send",
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
                        "/etc/profiles/per-user/pi/bin/onboard",
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

#[derive(Debug)]
enum ParseFingerError {
    BadParts,
    BadDirection,
    BadSize,
    BadEdge,
}

fn parse_finger(token: &str) -> Result<FingerPattern, ParseFingerError> {
    // Split on commas, ignore empty parts (e.g. trailing commas)
    let parts: Vec<&str> = token.split(',').filter(|s| !s.is_empty()).collect();

    if parts.len() < 2 || parts.len() > 3 {
        return Err(ParseFingerError::BadParts);
    }

    let dir = match parts[0] {
        "U" => Direction::Up,
        "UR" => Direction::UpRight,
        "UL" => Direction::UpLeft,
        "D" => Direction::Down,
        "DR" => Direction::DownRight,
        "DL" => Direction::DownLeft,
        "L" => Direction::Left,
        "R" => Direction::Right,
        "H" => Direction::Up, // Hold – direction is irrelevant later
        _ => return Err(ParseFingerError::BadDirection),
    };

    // Size (S,L)
    let size = match parts[1] {
        "S" => Size::S,
        "L" => Size::L,
        _ => return Err(ParseFingerError::BadSize),
    };

    // Edge (optional)
    let edge = if parts.len() == 3 {
        match parts[2] {
            "N" => Edge::None,
            "T" => Edge::Top,
            "R" => Edge::Right,
            "B" => Edge::Bottom,
            "L" => Edge::Left,
            "TL" => Edge::TopLeft,
            "TR" => Edge::TopRight,
            "BL" => Edge::BottomLeft,
            "BR" => Edge::BottomRight,
            _ => return Err(ParseFingerError::BadEdge),
        }
    } else {
        Edge::None
    };

    // Produce the FingerPattern – holds are represented as a Move with the given
    // size and origin; the direction field is ignored by the rest of the system.
    Ok(FingerPattern::new_move(dir, size, edge))
}

fn split_args(s: &str) -> Vec<String> {
    s.split_whitespace().map(|s| s.to_string()).collect()
}

fn parse_key_sequence(steps: Vec<String>) -> KeySequence {
    steps.into_iter().fold(KeySequence::default(), |seq, step| {
        seq.parse_step(&step)
            .expect("Invalid key sequence in configuration")
    })
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

// fn _parse_fingers(expr: &str) -> Result<Vec<FingerPattern>> {
//     Ok(expr
//         .split(';') .flat_map(|finger_expr| {
//             let mut chars = finger_expr.chars().peekable();
//             let repeat = repeat(&mut chars);
//             vec![FingerPattern::Hold { origin: Edge::None }; repeat]
//         })
//         .collect())
// }

// fn repeat(chars: &mut Peekable<Chars<'_>>) -> usize {
//     match chars.peek() {
//         Some(char) => {
//             if let Some(repeat) = char.to_digit(10) {
//                 let _ = chars.next();
//                 repeat as usize
//             } else {
//                 1
//             }
//         }
//         _ => 1,
//     }
// }

// mod tests {
//     use super::parse_fingers;
//     use touchscreen_gestures::{
//         error::Result,
//         touch::classifier::{Direction, Edge, FingerPattern, Size},
//     };
//     use Direction::{Down, DownLeft, DownRight, Left, Right, Up, UpLeft, UpRight};
//     use FingerPattern::{Hold, Move};

//     #[test]
//     fn test_parse_fingers_num_same() -> Result<()> {
//         assert_eq!(
//             parse_fingers("2 D L")?,
//             vec![
//                 Move {
//                     direction: Up,
//                     size: Size::L,
//                     origin: Edge::None
//                 },
//                 Move {
//                     direction: Up,
//                     size: Size::L,
//                     origin: Edge::None
//                 },
//             ]
//         );
//         Ok(())
//     }

//     #[test]
//     fn test_parse_fingers_list() -> Result<()> {
//         assert_eq!(
//             parse_fingers("U S L;U S R")?,
//             vec![
//                 Move { direction: Up, size: Size::L, origin: Edge::Left },
//                 Move { direction: Up, size: Size::L, origin: Edge::Right },
//             ]
//         );
//         Ok(())
//     }
// }
