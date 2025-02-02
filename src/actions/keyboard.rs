use std::collections::HashSet;
use std::time::Duration;

use crate::error::keyboard_init_failed;
use crate::error::{KeySequenceError, Result};

use evdev::{
    uinput::{VirtualDevice, VirtualDeviceBuilder},
    AttributeSet, KeyCode, KeyEvent,
};

pub type KeyResult<T> = std::result::Result<T, KeySequenceError>;

pub struct Keyboard {
    device: VirtualDevice,
}

impl Keyboard {
    pub fn new() -> Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        keys.insert(KeyCode::BTN_DPAD_UP);

        let mut device = VirtualDeviceBuilder::new()
            .map(|builder| builder.name("Fake Keyboard"))
            .and_then(|builder| builder.with_keys(&keys))
            .and_then(VirtualDeviceBuilder::build)
            .map_err(keyboard_init_failed)?;

        for path in device
            .enumerate_dev_nodes_blocking()
            .map_err(keyboard_init_failed)?
            .flatten()
        {
            println!("Available as {}", path.display());
        }

        Ok(Self { device })
    }

    pub fn press_sequence(&mut self, sequence: &KeySequence) {
        log::info!("Executing key sequence: {:?}", sequence);

        for step in &sequence.steps {
            // Press all modifier keys
            for modifier in &step.modifiers {
                self.key_down(modifier);
            }

            // Press all main keys
            for key in &step.keys {
                self.key_press(key);
            }

            // Release all modifier keys
            for modifier in &step.modifiers {
                self.key_up(modifier);
            }

            // Small delay between steps
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn key_down(&mut self, key: &KeyCode) {
        let event = *KeyEvent::new(*key, 1);
        self.device.emit(&[event]).unwrap();
    }

    fn key_up(&mut self, key: &KeyCode) {
        let event = *KeyEvent::new(*key, 0);
        self.device.emit(&[event]).unwrap();
    }

    fn key_press(&mut self, key: &KeyCode) {
        self.key_down(key);
        std::thread::sleep(Duration::from_millis(50));
        self.key_up(key);
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeySequence {
    /// Series of key combinations to be pressed in sequence
    steps: Vec<KeyStep>,
}

#[derive(Debug, Clone)]
struct KeyStep {
    /// Modifier keys that should be held during this step
    modifiers: HashSet<KeyCode>,
    /// The main key(s) to be pressed while modifiers are held
    keys: Vec<KeyCode>,
}

impl KeyStep {
    /// Parse a key combination string like "lctrl + lalt - a", "shift - b", or just "a"
    pub fn parse(input: &str) -> KeyResult<Self> {
        // First try splitting on '-' for modifier format
        let parts: Vec<&str> = input.split('-').collect();

        let (key_part, mods_part) = if parts.len() == 2 {
            (parts[1].trim(), parts[0].trim())
        } else if parts.len() == 1 {
            (parts[0].trim(), "")
        } else {
            return Err(KeySequenceError::InvalidFormat);
        };

        // Parse the main key
        let key = Self::parse_key(key_part)?;

        // Parse modifiers
        let modifiers = if mods_part.is_empty() {
            HashSet::new()
        } else {
            mods_part
                .split('+')
                .map(str::trim)
                .map(Self::parse_modifier)
                .collect::<KeyResult<HashSet<_>>>()?
        };

        Ok(Self {
            modifiers,
            keys: vec![key],
        })
    }

    fn parse_modifier(modifier: &str) -> KeyResult<KeyCode> {
        match modifier.to_lowercase().as_str() {
            "shift" => Ok(KeyCode::KEY_LEFTSHIFT),
            "lshift" => Ok(KeyCode::KEY_LEFTSHIFT),
            "rshift" => Ok(KeyCode::KEY_RIGHTSHIFT),
            "ctrl" => Ok(KeyCode::KEY_LEFTCTRL),
            "lctrl" => Ok(KeyCode::KEY_LEFTCTRL),
            "rctrl" => Ok(KeyCode::KEY_RIGHTCTRL),
            "alt" => Ok(KeyCode::KEY_LEFTALT),
            "lalt" => Ok(KeyCode::KEY_LEFTALT),
            "ralt" => Ok(KeyCode::KEY_RIGHTALT),
            "menu" => Ok(KeyCode::KEY_MENU),
            _ => Err(KeySequenceError::UnknownModifier(modifier.to_string())),
        }
    }

    fn parse_key(key: &str) -> KeyResult<KeyCode> {
        match key.to_lowercase().as_str() {
            // Letters
            "a" => Ok(KeyCode::KEY_A),
            "b" => Ok(KeyCode::KEY_B),
            "c" => Ok(KeyCode::KEY_C),
            "d" => Ok(KeyCode::KEY_D),
            "e" => Ok(KeyCode::KEY_E),
            "f" => Ok(KeyCode::KEY_F),
            "g" => Ok(KeyCode::KEY_G),
            "h" => Ok(KeyCode::KEY_H),
            "i" => Ok(KeyCode::KEY_I),
            "j" => Ok(KeyCode::KEY_J),
            "k" => Ok(KeyCode::KEY_K),
            "l" => Ok(KeyCode::KEY_L),
            "m" => Ok(KeyCode::KEY_M),
            "n" => Ok(KeyCode::KEY_N),
            "o" => Ok(KeyCode::KEY_O),
            "p" => Ok(KeyCode::KEY_P),
            "q" => Ok(KeyCode::KEY_Q),
            "r" => Ok(KeyCode::KEY_R),
            "s" => Ok(KeyCode::KEY_S),
            "t" => Ok(KeyCode::KEY_T),
            "u" => Ok(KeyCode::KEY_U),
            "v" => Ok(KeyCode::KEY_V),
            "w" => Ok(KeyCode::KEY_W),
            "x" => Ok(KeyCode::KEY_X),
            "y" => Ok(KeyCode::KEY_Y),
            "z" => Ok(KeyCode::KEY_Z),

            // Numbers
            "0" => Ok(KeyCode::KEY_0),
            "1" => Ok(KeyCode::KEY_1),
            "2" => Ok(KeyCode::KEY_2),
            "3" => Ok(KeyCode::KEY_3),
            "4" => Ok(KeyCode::KEY_4),
            "5" => Ok(KeyCode::KEY_5),
            "6" => Ok(KeyCode::KEY_6),
            "7" => Ok(KeyCode::KEY_7),
            "8" => Ok(KeyCode::KEY_8),
            "9" => Ok(KeyCode::KEY_9),

            // Function keys
            "f1" => Ok(KeyCode::KEY_F1),
            "f2" => Ok(KeyCode::KEY_F2),
            "f3" => Ok(KeyCode::KEY_F3),
            "f4" => Ok(KeyCode::KEY_F4),
            "f5" => Ok(KeyCode::KEY_F5),
            "f6" => Ok(KeyCode::KEY_F6),
            "f7" => Ok(KeyCode::KEY_F7),
            "f8" => Ok(KeyCode::KEY_F8),
            "f9" => Ok(KeyCode::KEY_F9),
            "f10" => Ok(KeyCode::KEY_F10),
            "f11" => Ok(KeyCode::KEY_F11),
            "f12" => Ok(KeyCode::KEY_F12),

            // Special keys
            "esc" | "escape" => Ok(KeyCode::KEY_ESC),
            "enter" | "return" => Ok(KeyCode::KEY_ENTER),
            "space" => Ok(KeyCode::KEY_SPACE),
            "tab" => Ok(KeyCode::KEY_TAB),
            "backspace" => Ok(KeyCode::KEY_BACKSPACE),
            "delete" | "del" => Ok(KeyCode::KEY_DELETE),
            "insert" | "ins" => Ok(KeyCode::KEY_INSERT),
            "home" => Ok(KeyCode::KEY_HOME),
            "end" => Ok(KeyCode::KEY_END),
            "pageup" | "pgup" => Ok(KeyCode::KEY_PAGEUP),
            "pagedown" | "pgdn" => Ok(KeyCode::KEY_PAGEDOWN),

            // Arrow keys
            "up" => Ok(KeyCode::KEY_UP),
            "down" => Ok(KeyCode::KEY_DOWN),
            "left" => Ok(KeyCode::KEY_LEFT),
            "right" => Ok(KeyCode::KEY_RIGHT),

            // Common punctuation
            "." | "period" => Ok(KeyCode::KEY_DOT),
            "," | "comma" => Ok(KeyCode::KEY_COMMA),
            "/" | "slash" => Ok(KeyCode::KEY_SLASH),
            "\\" | "backslash" => Ok(KeyCode::KEY_BACKSLASH),
            ";" | "semicolon" => Ok(KeyCode::KEY_SEMICOLON),
            "'" | "apostrophe" => Ok(KeyCode::KEY_APOSTROPHE),
            "[" | "leftbracket" => Ok(KeyCode::KEY_LEFTBRACE),
            "]" | "rightbracket" => Ok(KeyCode::KEY_RIGHTBRACE),
            "-" | "minus" => Ok(KeyCode::KEY_MINUS),
            "=" | "equals" => Ok(KeyCode::KEY_EQUAL),
            "`" | "grave" => Ok(KeyCode::KEY_GRAVE),

            _ => Err(KeySequenceError::UnknownKey(key.to_string())),
        }
    }
}

impl KeySequence {
    /// Parse a string into a KeyStep and add it to the sequence
    pub fn parse_step(self, input: &str) -> KeyResult<Self> {
        let step = KeyStep::parse(input)?;
        Ok(self.add_step_owned(step))
    }

    /// Add a KeyStep directly
    fn add_step_owned(mut self, step: KeyStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add a new step with modifiers and keys
    pub fn add_step(mut self, modifiers: &[KeyCode], keys: &[KeyCode]) -> Self {
        self.steps.push(KeyStep {
            modifiers: modifiers.iter().copied().collect(),
            keys: keys.to_vec(),
        });
        self
    }

    /// Add a simple key press without modifiers
    pub fn press(self, key: KeyCode) -> Self {
        self.add_step(&[], &[key])
    }

    /// Add modifier keys to be held while pressing the main key
    pub fn with_mods(self, mods: &[KeyCode], key: KeyCode) -> Self {
        self.add_step(mods, &[key])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::KeySequenceError;

    #[test]
    fn test_basic_key_sequence() -> Result<()> {
        let seq = KeySequence::default().parse_step("a")?;

        assert_eq!(seq.steps.len(), 1);
        assert!(seq.steps[0].modifiers.is_empty());
        assert_eq!(seq.steps[0].keys, vec![KeyCode::KEY_A]);
        Ok(())
    }

    #[test]
    fn test_single_modifier() -> Result<()> {
        let seq = KeySequence::default().parse_step("ctrl - x")?;

        assert_eq!(seq.steps.len(), 1);
        assert_eq!(seq.steps[0].modifiers.len(), 1);
        assert!(seq.steps[0].modifiers.contains(&KeyCode::KEY_LEFTCTRL));
        assert_eq!(seq.steps[0].keys, vec![KeyCode::KEY_X]);
        Ok(())
    }

    #[test]
    fn test_multiple_modifiers() -> Result<()> {
        let seq = KeySequence::default().parse_step("lctrl + lalt - delete")?;

        assert_eq!(seq.steps.len(), 1);
        assert_eq!(seq.steps[0].modifiers.len(), 2);
        assert!(seq.steps[0].modifiers.contains(&KeyCode::KEY_LEFTCTRL));
        assert!(seq.steps[0].modifiers.contains(&KeyCode::KEY_LEFTALT));
        assert_eq!(seq.steps[0].keys, vec![KeyCode::KEY_DELETE]);
        Ok(())
    }

    #[test]
    fn test_multiple_steps() -> Result<()> {
        let seq = KeySequence::default()
            .parse_step("shift - a")?
            .parse_step("ctrl + alt - delete")?;

        assert_eq!(seq.steps.len(), 2);

        // First step
        assert_eq!(seq.steps[0].modifiers.len(), 1);
        assert!(seq.steps[0].modifiers.contains(&KeyCode::KEY_LEFTSHIFT));
        assert_eq!(seq.steps[0].keys, vec![KeyCode::KEY_A]);

        // Second step
        assert_eq!(seq.steps[1].modifiers.len(), 2);
        assert!(seq.steps[1].modifiers.contains(&KeyCode::KEY_LEFTCTRL));
        assert!(seq.steps[1].modifiers.contains(&KeyCode::KEY_LEFTALT));
        assert_eq!(seq.steps[1].keys, vec![KeyCode::KEY_DELETE]);
        Ok(())
    }

    #[test]
    fn test_special_keys() -> Result<()> {
        let seq = KeySequence::default()
            .parse_step("- escape")?
            .parse_step("- f11")?
            .parse_step("- pageup")?;

        assert_eq!(seq.steps.len(), 3);
        assert_eq!(seq.steps[0].keys, vec![KeyCode::KEY_ESC]);
        assert_eq!(seq.steps[1].keys, vec![KeyCode::KEY_F11]);
        assert_eq!(seq.steps[2].keys, vec![KeyCode::KEY_PAGEUP]);
        Ok(())
    }

    #[test]
    fn test_error_invalid_format() {
        let result = KeySequence::default().parse_step("ctrl - alt - delete");
        assert_eq!(result.unwrap_err(), KeySequenceError::InvalidFormat);
    }

    #[test]
    fn test_error_unknown_modifier() {
        let result = KeySequence::default().parse_step("invalid + ctrl - a");
        assert_eq!(
            result.unwrap_err(),
            KeySequenceError::UnknownModifier("invalid".to_string())
        );
    }

    #[test]
    fn test_error_unknown_key() {
        let result = KeySequence::default().parse_step("ctrl - invalid");
        assert_eq!(
            result.unwrap_err(),
            KeySequenceError::UnknownKey("invalid".to_string())
        );
    }
}
