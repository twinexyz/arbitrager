#[derive(Clone, Debug)]
pub enum AggregatorError {
    ELFFileNotFound(String),
    FailToReadELF,
    VerificationFailed,
    ProofParsingFailed,
    InvalidSender(String),
    JsonRPCServerError(String),
    DBError(String),
    SubmitTransactionFailed(String),
    PosterError(String),
    Custom(String),
}

impl std::error::Error for AggregatorError {}

impl std::fmt::Display for AggregatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregatorError::ELFFileNotFound(_) => write!(f, "ELF File Not found"),
            AggregatorError::FailToReadELF => write!(f, "Failed to read ELF file"),
            AggregatorError::VerificationFailed => write!(f, "Proof  Verification Failed"),
            AggregatorError::Custom(e) => write!(f, "{e}"),
            AggregatorError::InvalidSender(e) => write!(f, "Invalid sender: {e}"),
            AggregatorError::ProofParsingFailed => write!(f, "Failed to parse proof"),
            AggregatorError::JsonRPCServerError(e) => write!(f, "{e:?}"),
            AggregatorError::SubmitTransactionFailed(e) => write!(f, "{e:?}"),
            AggregatorError::DBError(e) => write!(f, "{e:?}"),
            AggregatorError::PosterError(e) => write!(f, "{e:?}"),
        }
    }
}
