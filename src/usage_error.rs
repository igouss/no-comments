#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsageError {
    pub message: String,
}

impl UsageError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> UsageError {
        UsageError {
            message: message.into(),
        }
    }
}
