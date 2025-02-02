use std::io;

use thiserror::Error;
use xrandr::XrandrError;

pub type Result<T> = std::result::Result<T, GesturesError>;

#[derive(Error, Debug)]
pub enum GesturesError {
    #[error("Accelerometer sensor not found")]
    AccelerometerMissing,

    #[error("Failed to read accelerometer data")]
    AccelerometerFailed {
        #[source]
        source: Box<dyn std::error::Error>,
    },

    #[error("I/O error in keyboard")]
    KeyboardInitFailed {
        #[source]
        source: Box<dyn std::error::Error>,
    },

    #[error("xrandr error")]
    XRandError {
        #[from]
        #[source]
        source: XrandrError,
    },

    #[error("Invalid key sequence: {0}")]
    KeySequenceInvalid(#[from] KeySequenceError),
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum KeySequenceError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Unknown modifier key: {0}")]
    UnknownModifier(String),

    #[error("Unknown key: {0}")]
    UnknownKey(String),
}

impl From<zbus::Error> for GesturesError {
    fn from(err: zbus::Error) -> Self {
        GesturesError::AccelerometerFailed {
            source: Box::new(err),
        }
    }
}

pub(crate) fn keyboard_init_failed(source: io::Error) -> GesturesError {
    GesturesError::KeyboardInitFailed {
        source: Box::new(source),
    }
}
