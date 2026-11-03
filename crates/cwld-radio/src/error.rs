use cwld_types::TxPolicy;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RigError {
    /// The transmit guard refused the command in the current policy.
    #[error("command {command} is blocked in policy {policy:?}")]
    Blocked { command: &'static str, policy: TxPolicy },
    #[error("invalid argument: {0}")]
    InvalidArgument(String),
    #[error("radio is not connected")]
    NotConnected,
    #[error("radio error: {0}")]
    Io(String),
}

impl RigError {
    /// A short, stable code for the user interface.
    pub fn code(&self) -> &'static str {
        match self {
            RigError::Blocked { .. } => "blocked",
            RigError::InvalidArgument(m) => match m.as_str() {
                "word_too_long" => "word_too_long",
                "message_too_long" => "message_too_long",
                "empty text" => "empty_text",
                "port" => "port_missing",
                _ => "invalid_argument",
            },
            RigError::NotConnected => "not_connected",
            RigError::Io(_) => "io",
        }
    }
}
