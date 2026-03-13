// MVI Error Types

use std::fmt;

#[derive(Debug, Clone)]
pub enum MviError {
    StateError(String),
    IntentError(String),
    Unknown(String),
}

impl fmt::Display for MviError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StateError(msg) => write!(f, "State error: {}", msg),
            Self::IntentError(msg) => write!(f, "Intent error: {}", msg),
            Self::Unknown(msg) => write!(f, "Unknown error: {}", msg),
        }
    }
}

impl std::error::Error for MviError {}

pub type MviResult<T> = Result<T, MviError>;
