#[derive(Clone, Debug)]
pub enum ArbitragerError {
    ELFFileNotFound(String),
    FailToReadELF,
    VerificationFailed,
    ProofParsingFailed,
    InvalidSender(String),
    Custom(String),
}

impl std::error::Error for ArbitragerError {}

impl std::fmt::Display for ArbitragerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArbitragerError::ELFFileNotFound(_) => write!(f, "ELF File Not found"),
            ArbitragerError::FailToReadELF => write!(f, "Failed to read ELF file"),
            ArbitragerError::VerificationFailed => write!(f, "Proof  Verification Failed"),
            ArbitragerError::Custom(e) => write!(f, "{e}"),
            ArbitragerError::InvalidSender(e) => write!(f, "Invalid sender: {e}"),
            ArbitragerError::ProofParsingFailed => write!(f, "Failed to parse proof"),
        }
    }
}
