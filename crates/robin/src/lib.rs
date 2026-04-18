pub mod pdf;

#[derive(Debug, thiserror::Error)]
pub enum RobinError {
    #[error("lopdf error: {0}")]
    LopdfError(#[from] lopdf::Error),
    
    #[error("invalid margin, must be between 0.0 and 1.0")]
    InvalidMargin,
    
    #[error("pdf text not found")]
    TextSpansNotFound,
}