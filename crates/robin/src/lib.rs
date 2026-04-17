pub mod pdf;

#[derive(Debug, thiserror::Error)]
pub enum RobinError {
    #[error("lopdf error: {0}")]
    LopdfError(#[from] lopdf::Error),
}