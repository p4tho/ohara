pub mod pdf;

#[derive(thiserror::Error, Debug)]
pub enum RobinError {
    #[error("lopdf error: {0}")]
    LopdfError(#[from] lopdf::Error),

    #[error("invalid bookmark title")]
    InvalidTitle
}